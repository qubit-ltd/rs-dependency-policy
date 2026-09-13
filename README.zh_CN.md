# rs-dependency-policy

[![Rust CI](https://github.com/qubit-ltd/rs-dependency-policy/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-dependency-policy/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-dependency-policy/coverage-badge.json)](https://qubit-ltd.github.io/rs-dependency-policy/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-dependency-policy.svg?color=blue)](https://crates.io/crates/qubit-dependency-policy)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`rs-dependency-policy` 用于为多个 Rust 库、私有 crate 和独立应用建立并执行统一的第三方依赖基线。它帮助维护者确认：同一个外部 crate 应使用什么版本约束，哪些仓库当前存在不一致。

工具不绑定任何组织的 crate 命名规则。内部 crate 的识别方式由调用方传入，工具本身不写死组织名称。

## 安装

开发阶段可以从源码安装 CLI：

```bash
git clone https://github.com/qubit-ltd/rs-dependency-policy.git
cd rs-dependency-policy
cargo install --path .
```

也可以在源码目录直接使用下文的 `cargo run --` 命令。随仓库提供的脚本需要 Bash、Cargo，以及用于交互选择版本的 `jq`。

## 快速开始：创建 baseline

如果需要为多个仓库目录建立一条第三方依赖基线，在本仓库运行交互式脚本：

```bash
./scripts/create-baseline.sh \
  --root /work/rust-common \
  --root /work/rust-platform \
  --internal-prefix acme- \
  --internal-prefix acme_rs- \
  --release v2026.09.13
```

每个 `--root` 可以是包含 `Cargo.toml` 的 Rust 项目目录，也可以是项目父目录；传入父目录时，脚本会扫描其中一级子目录下的 Rust 项目。脚本会盘点直接依赖声明和 Cargo 解析后的依赖图；当外部 crate 的版本要求不一致时，会要求操作者选择：

```text
依赖 criterion 存在多个声明版本：
  1) ^0.8
  2) ^0.5
选择 [1-2]（默认 1，输入 q 放弃）：
```

候选 baseline 默认写入 `policy/baselines/<release>.toml`。请先审核后再提交。脚本不会修改被扫描的业务仓库；manifest 无法解析的项目会被跳过并显示路径，需另行修复。

`path` 和 `workspace` 依赖始终视为内部依赖。registry 或 Git 依赖默认视为第三方依赖；若某些已发布 crate 仍属于内部生态，可重复传入 `--internal-prefix` 将其排除。没有命名空间时不传该参数即可。

## 只盘点，不做版本决策

需要先产出供团队评审的 JSON 原始数据时，使用非交互脚本：

```bash
./scripts/bootstrap-baseline.sh \
  --root /work/rust-common \
  --root /work/rust-platform \
  --output /tmp/dependency-inventory.json
```

等价的 CLI 只接收明确的项目根目录：

```bash
cargo run -- inventory \
  --root /work/rust-common/rs-example \
  --root /work/rust-platform/rs-service \
  --format markdown
```

盘点结果包含 normal/build/dev 直接依赖、optional 属性、workspace 包名、解析后的包版本，以及直接版本约束冲突。它是建立 baseline 的依据，不会自动成为已批准的策略。

## 接入已审核的 baseline

在 policy 仓库提交经过审核的 release 后，为每个受治理项目添加 `.infra/dep/policy.toml`。当前版本支持本地 `file://` 形式的策略源：

```toml
format = 1

[baseline]
name = "organization-third-party"
source = "file:///absolute/path/to/rs-dependency-policy"
revision = "0123456789abcdef0123456789abcdef01234567"
release = "v2026.09.13"

[project]
profile = "library" # 已锁定依赖图的应用使用 "application"
```

`revision` 应填写审核该 release 时的完整 Git commit SHA。当前版本尚未实现远程 Git 策略源下载和本地策略源 revision 校验，因此本地 `file://` 是已支持的执行方式。

## 检查、报告与安全同步

检查一个项目是否符合它选择的 baseline：

```bash
cargo run -- --project /work/rs-example check
```

输出人类可读或 JSON 格式的报告：

```bash
cargo run -- --project /work/rs-example report --format markdown
cargo run -- --project /work/rs-example report --format json
```

先生成安全的版本修改计划，确认后再执行：

```bash
cargo run -- --project /work/rs-example sync --dry-run
cargo run -- --project /work/rs-example sync
```

当前同步仅会修改根 `Cargo.toml` 的 `[dependencies]` 中纯字符串版本声明，例如 `serde = "1.0"`。inline table、别名依赖、target-specific 依赖、workspace 依赖及 `Cargo.lock` 更新都需要人工审核，不会被自动修改。

## 当前能力与边界

- 多项目 JSON/Markdown inventory；
- 根据外部依赖冲突交互生成候选 baseline；
- versioned baseline 中的 library/application profile；
- 直接版本约束、禁止 resolved 版本、可选单版本 resolved 图检查；
- 带 dry-run 的保守同步。

当前尚不支持 Git 拉取 baseline、校验本地 source 与记录 revision 一致性、阻止未登记依赖、应用 exception 文件，以及大范围自动修改 Cargo manifest 或 lockfile。

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-dependency-policy](https://github.com/qubit-ltd/rs-dependency-policy)
