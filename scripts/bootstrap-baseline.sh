#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_dir=$(cd -- "${script_dir}/.." && pwd)
output="${repo_dir}/target/dependency-inventory.json"
roots=()

usage() {
    cat <<'EOF'
用法：bootstrap-baseline.sh --root <项目目录> [--root <项目目录> ...] [--output <文件>]

只生成候选 inventory，不会修改任何业务仓库或正式 baseline。
EOF
}

while (($# > 0)); do
    case "$1" in
        --root)
            (($# >= 2)) || { usage >&2; exit 2; }
            roots+=("$2")
            shift 2
            ;;
        --output)
            (($# >= 2)) || { usage >&2; exit 2; }
            output="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf '未知参数：%s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

((${#roots[@]} > 0)) || {
    printf '至少需要一个 --root；脚本不会猜测组织仓库范围。\n' >&2
    usage >&2
    exit 2
}

for root in "${roots[@]}"; do
    [[ -f "${root}/Cargo.toml" ]] || {
        printf '不是 Rust 项目根目录（缺少 Cargo.toml）：%s\n' "$root" >&2
        exit 2
    }
done

mkdir -p "$(dirname -- "$output")"
args=(run --quiet -- inventory)
for root in "${roots[@]}"; do
    args+=(--root "$root")
done
args+=(--format json --output "$output")

(cd "$repo_dir" && cargo "${args[@]}")
printf '候选 inventory 已生成：%s\n' "$output"
printf '请审核 conflicts 和版本选择后，再把结果转为带 revision 的正式 baseline。\n'
