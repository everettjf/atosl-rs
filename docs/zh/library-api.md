---
title: 库 API
layout: default
parent: 中文文档
nav_order: 9
---

# 库 API（Rust）

`atosl` 同时也是一个 crate。如果你正在用 Rust 构建一个崩溃处理工具，可以直接调用符号化引擎，从而获得结构化的结果，而无需解析终端输出。

## 添加依赖

```toml
[dependencies]
atosl = "0.2"
```

## 符号化

`SymbolizeOptions` 实现了 `Default`，因此你只需设置你关心的字段，其余字段通过 `..Default::default()` 回退：

```rust
use atosl::{OutputFormat, SymbolizeOptions};

let report = atosl::symbolize_path(&SymbolizeOptions {
    object_path: "MyApp.app.dSYM".into(),
    load_address: 0x1_0000_0000,
    addresses: vec![0x1_0000_1234],
    arch: Some("arm64".to_string()),
    format: OutputFormat::Json,
    ..Default::default()
})?;

for outcome in &report.frames {
    println!("{outcome:?}");
}
# Ok::<(), anyhow::Error>(())
```

使用 `..Default::default()` 还能让你的代码在未来版本向 `SymbolizeOptions` 添加新的可选字段时仍然可以编译通过。

## 选项

| 字段 | 类型 | 用途 |
| --- | --- | --- |
| `object_path` | `PathBuf` | 对象、dSYM 载荷、`.dSYM` bundle 或目录 |
| `load_address` | `u64` | 镜像加载地址（参见[地址模式](address-modes)） |
| `addresses` | `Vec<u64>` | 要解析的地址 |
| `file_offsets` | `bool` | 旧式 `-f` 模式（`address − load_address`） |
| `inline_frames` | `bool` | 在文本渲染中展开内联帧 |
| `arch` | `Option<String>` | 按架构选择 fat 切片 |
| `uuid` | `Option<String>` | 按 UUID 选择 fat 切片 / 目录中的文件 |
| `format` | `OutputFormat` | CLI 输出器使用的输出格式 |
| `input` | `Option<PathBuf>` | 从文件读取地址 |
| `debug_dirs` | `Vec<PathBuf>` | 独立 ELF 调试文件的额外根目录 |
| `verbose` | `bool` | 解析器诊断信息 |

## 结果

`symbolize_path` 返回一个 `SymbolizeReport`：

```rust
pub struct SymbolizeReport {
    pub object_path: String,
    pub object_name: String,
    pub selected_slice: Option<SelectedSlice>,
    pub frames: Vec<SymbolizeOutcome>,
}
```

每个 `SymbolizeOutcome` 要么是 `Resolved(SymbolizedFrame)`，要么是 `Unresolved { requested_address, error }`。`SymbolizedFrame` 携带符号、产生该符号的解析器（`dwarf` 或 `symbol_table`）、可选的源码 `location`，以及 `inlined_by` 链。这些与 [JSON 字段参考](output-formats)直接对应。

> 无论 `inline_frames` 标志如何，报告的 `inlined_by` 中始终包含完整的内联链——该标志只影响 CLI 的文本渲染。

## 稳定性

`SymbolizeOptions` 派生 `Default` 意味着可以添加新的可选字段，而不会破坏使用 `..Default::default()` 的调用方。参见项目的[关于 API 兼容性的发布说明](https://github.com/everettjf/atosl-rs/blob/main/RELEASING.md#public-api-compatibility)。
