# rs-dependency-policy

[![Rust CI](https://github.com/qubit-ltd/rs-dependency-policy/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-dependency-policy/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/qubit-dependency-policy.svg?color=blue)](https://crates.io/crates/qubit-dependency-policy)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向跨仓库 Rust 项目的第三方依赖基线治理工具。项目在 .infra/dep/policy.toml 中选择不可变的策略版本；CLI 检查 manifest 和解析后的依赖图，不要求使用 Git submodule。

## 适用对象

适用于公开发布的库、私有 crate 和独立应用，用来在多个仓库间维护可复现的依赖规则。

## 安装

~~~bash
cargo install qubit-dependency-policy
~~~

## 最小用法

~~~bash
cargo dependency-policy check --project .
~~~

当前版本支持配置加载、本地策略源、Cargo metadata 评估、JSON/Markdown 报告和保守的直接版本同步。Git 策略源获取、全仓库迁移自动化和 rs-ci 集成将在独立阶段完成。

## 建立基线前的盘点

基线应由一次可审查的全量盘点产生，而不是从单个仓库猜测。可重复传入多个项目根目录：

~~~bash
cargo dependency-policy inventory \
  --root ../rust-common/rs-value \
  --root ../rust-platform/rs-reflect \
  --format json --output dependency-inventory.json
~~~

盘点结果同时包含各 workspace 的直接声明、Cargo 解析后的依赖图，以及存在多个声明版本的冲突列表。冲突项目必须先完成统一版本决策，再人工审核并提交到共享 baseline；工具不会把未经审核的盘点结果直接当作策略。

## 测试

~~~bash
cargo test
cargo test --all-features
./ci-check.sh
./coverage.sh
~~~

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 ./align-ci.sh 格式化代码，运行 ./ci-check.sh 对齐 CI 要求。

内部 crate 命名空间通过调用参数传入，不由工具写死。例如 Qubit 项目可增加
`--internal-prefix qubit- --internal-prefix rs-`；其它组织应替换为自己的前缀，或不传该参数。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-dependency-policy](https://github.com/qubit-ltd/rs-dependency-policy)
