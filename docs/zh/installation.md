---
title: 安装
layout: default
parent: 中文文档
nav_order: 1
---

# 安装

`atosl` 是一个自包含的单文件二进制程序。有两种获取方式。

## 从 crates.io 安装

```bash
cargo install atosl
```

这会构建 `atosl` 二进制并安装到 `~/.cargo/bin`。请确保该目录在你的 `PATH` 中。

## 从源码构建

```bash
git clone https://github.com/everettjf/atosl-rs.git
cd atosl-rs
cargo build --release
./target/release/atosl --help
```

release 版本的二进制会写入 `target/release/atosl`。

## 环境要求

- 较新的稳定版 Rust 工具链（该 crate 设置了 `rust-version = "1.85"`）。
- 运行时不依赖 Xcode 或 Apple 工具链。`atosl` 直接读取 Mach-O、DWARF 和 ELF。

## 验证安装

```bash
atosl --version
atosl --help
```

`--help` 会打印每个标志及其简短说明，是你学习该工具时最快的参考。

## 下一步

继续阅读[快速上手](getting-started)，完整地运行你的第一次符号化。
