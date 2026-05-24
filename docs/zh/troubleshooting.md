---
title: 排错与限制
layout: default
parent: 中文文档
nav_order: 10
---

# 排错与限制

## 读懂常见消息

### `N/A - failed to search symbol table`

查找地址落在了所有符号之外。通常是地址或加载地址有误。请重新核对[地址模式](address-modes)：对于崩溃报告，你应使用**默认模式**，配合 Binary Images 中该镜像的加载地址。

### `address 0x… is smaller than load address 0x…`

你减去的值超过了地址本身所能容纳的范围。你传入的值低于加载地址，所以它不可能是该镜像的运行时地址。要么是你用错了加载地址，要么该值其实是一个文件偏移——这种情况下请使用 `-l 0 <offset>`（参见[地址模式](address-modes)）。

### `fat Mach-O contains multiple slices`

你指向了一个通用（universal）二进制却没有选择切片。请添加 `--arch` 或 `--uuid`。参见 [Fat 二进制与切片](fat-binaries)。

### `no binary or dSYM under <dir> matched uuid <uuid>`

目录搜索没有找到具有该 UUID/build-id 的文件。请确认 UUID（来自崩溃报告的 Binary Images）正确，并确认该确切构建对应的 dSYM 确实在该目录中。

## 已知限制

### Mach-O 调试映射（`N_OSO`）

当你使用 `-g` 构建但**没有**运行 `dsymutil` 时，可执行文件只保留了指向原始 `.o` 文件的 `N_OSO` stab 条目；行表 DWARF 存在于那些目标文件中。Apple 的 `atos` 会遍历该调试映射以恢复源码行。`atosl` **不会**跟随调试映射，因此对于这样的二进制，它会回退到符号表（`symbol + offset`）。

**修复方法：** 让 `atosl` 指向一个生成好的 `.dSYM`（运行 `dsymutil`），或指向一个直接嵌入了 DWARF 的对象。这样源码位置就能像 `atos` 一样精确地解析出来。

### 源码路径以完整形式打印

除非给定 `-fullPath`，否则 Apple `atos` 只打印文件名。`atosl` 始终打印 DWARF 行表中记录的路径。这只是外观上的差异；文件和行号是相同的。

### 适用范围

- `atosl` 并非 `atos` 的一比一克隆。Mach-O 工作流是主要的设计目标；其他格式在存在符号时表现最佳。
- 符号化质量取决于目标文件中的符号与 DWARF 数据。任何工具都无法恢复文件中不存在的信息。
- 真实崩溃日志的*摄取*（解析完整的 `.crash`/`.ips` 文件）不在范围之内——`atosl` 负责符号化地址；提取地址则由你自行完成。

## 仍然卡住？

使用 `-v/--verbose` 运行，在标准错误（stderr）上查看解析器诊断信息（选用了哪个解析器、每个帧的查找地址）：

```bash
atosl -v -o app.dSYM -l 0x100000000 --arch arm64 0x100001234
```

如果某个现象看起来像 bug，请提交一个 issue，附上命令、`-v` 输出以及它与 `atos` 的差异之处：
<https://github.com/everettjf/atosl-rs/issues>。
