#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_dir=$(cd -- "${script_dir}/.." && pwd)
release="v$(date +%Y.%m.%d)"
output="${repo_dir}/policy/baselines/${release}.toml"
roots=()
internal_prefixes=()

usage() {
    cat <<'EOF'
用法：create-baseline.sh --root <项目目录> [--root <项目目录> ...] [选项]

选项：
  --release <版本>   baseline release，默认当前日期 vYYYY.MM.DD
  --output <文件>    输出候选 baseline TOML
  --internal-prefix <前缀>  声明内部 crate 命名空间前缀（可重复；默认不排除任何 registry crate）
  --help             显示帮助

脚本会逐项询问冲突依赖选择。它只生成候选文件，不修改业务仓库。
EOF
}

while (($# > 0)); do
    case "$1" in
        --root) (($# >= 2)) || { usage >&2; exit 2; }; roots+=("$2"); shift 2 ;;
        --release) (($# >= 2)) || { usage >&2; exit 2; }; release="$2"; shift 2 ;;
        --output) (($# >= 2)) || { usage >&2; exit 2; }; output="$2"; shift 2 ;;
        --internal-prefix) (($# >= 2)) || { usage >&2; exit 2; }; internal_prefixes+=("$2"); shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) printf '未知参数：%s\n' "$1" >&2; usage >&2; exit 2 ;;
    esac
done

((${#roots[@]} > 0)) || { printf '至少需要一个 --root。\n' >&2; usage >&2; exit 2; }
command -v jq >/dev/null || { printf '需要 jq 才能交互选择依赖版本。\n' >&2; exit 2; }

expanded=()
for root in "${roots[@]}"; do
    if [[ -f "${root}/Cargo.toml" ]]; then
        expanded+=("$root")
        continue
    fi
    found=0
    for manifest in "$root"/*/Cargo.toml; do
        [[ -f "$manifest" ]] || continue
        expanded+=("${manifest%/Cargo.toml}")
        found=1
    done
    ((found)) || { printf '不是 Rust 项目目录或一级父目录：%s\n' "$root" >&2; exit 2; }
done
roots=("${expanded[@]}")

valid_roots=()
for root in "${roots[@]}"; do
    if cargo metadata --manifest-path "${root}/Cargo.toml" --format-version 1 >/dev/null 2>&1; then
        valid_roots+=("$root")
    else
        printf '跳过无法解析的项目：%s（请单独修复 Cargo 版本/path 依赖）\n' "$root" >&2
    fi
done
roots=("${valid_roots[@]}")
((${#roots[@]} > 0)) || { printf '没有可扫描的 Rust 项目。\n' >&2; exit 2; }

inventory=$(mktemp)
trap 'rm -f "$inventory"' EXIT
args=(run --quiet -- inventory)
for root in "${roots[@]}"; do args+=(--root "$root"); done
args+=(--format json --output "$inventory")
(cd "$repo_dir" && cargo "${args[@]}")

mkdir -p "$(dirname -- "$output")"
{
    printf 'format = 1\nrelease = "%s"\n\n[profiles.library]\n' "$release"
    printf '# Generated from inventory; review before publishing.\n'
    while IFS= read -r name; do
        internal=0
        for prefix in "${internal_prefixes[@]}"; do
            if [[ "$name" == "${prefix}"* ]]; then internal=1; break; fi
        done
        ((internal)) && continue
        mapfile -t choices < <(jq -r --arg n "$name" '.direct_requirements[$n][]' "$inventory")
        selected="${choices[0]}"
        if ((${#choices[@]} > 1)); then
            printf '\n依赖 %s 存在多个声明版本：\n' "$name" >&2
            for i in "${!choices[@]}"; do printf '  %d) %s\n' "$((i + 1))" "${choices[$i]}" >&2; done
            [[ -r /dev/tty ]] || { printf '无法访问交互终端，请在终端运行后选择版本。\n' >&2; exit 2; }
            while true; do
                if ! read -r -p "选择 [1-${#choices[@]}]（默认 1，输入 q 放弃）：" answer </dev/tty; then
                    printf '\n未检测到交互输入；请在终端运行并选择版本，未生成正式 baseline。\n' >&2
                    exit 2
                fi
                answer=${answer:-1}
                [[ "$answer" == q ]] && { printf '用户取消，未写入 baseline。\n' >&2; exit 1; }
                [[ "$answer" =~ ^[0-9]+$ && "$answer" -ge 1 && "$answer" -le "${#choices[@]}" ]] && { selected="${choices[$((answer - 1))]}"; break; }
                printf '请输入有效编号。\n' >&2
            done
        fi
        printf '[profiles.library.direct."%s"]\nrequirement = "%s"\n\n' "$name" "$selected"
    done < <(jq -r '.direct_requirements | keys[]' "$inventory")
    printf '[profiles.application]\n\n# Resolved graph rules are intentionally left for review.\n'
} >"$output"

printf '候选 baseline 已生成：%s\n' "$output"
printf '请审核 direct 版本、resolved 规则和 application 配置后再提交。\n'
