---
title: 地址模式
layout: default
parent: 中文文档
nav_order: 3
---

# 地址模式

符号化中最常见的困惑来源就是*一个地址究竟意味着什么*。`atosl` 会根据你传入的标志来解释每个输入地址。

## 查找地址（lookup address）

在内部，每一次符号化都会解析一个**静态链接期 VM 地址**——即记录在 DWARF 行表（line table）和符号表中的地址（对于典型的 arm64 可执行文件，这些地址从 `0x100000000` 开始）。这些标志决定了你的输入如何被转换为该查找地址。

| 模式 | 标志 | 查找地址 | 典型用途 |
| --- | --- | --- | --- |
| 加载地址（默认） | `-l <load>` | `address − load_address + __TEXT vmaddr` | 来自崩溃报告的运行时/虚拟地址 |
| 等价于 `atos -offset` | `-l 0 <off>` | `off + __TEXT vmaddr` | 相对于镜像 `__TEXT` 基址的文件偏移 |
| 文件偏移（旧式 `-f`） | `-f -l <load>` | `address − load_address` | 跳过 `__TEXT` 重定基的向后兼容模式 |

## 默认模式：运行时地址

这是你处理崩溃报告时所使用的模式。你需要传入镜像被映射到的**加载地址**，以及崩溃记录下的**运行时地址**。

```bash
atosl -o MyApp.app.dSYM -l 0x104f80000 0x104f81234
```

`atosl` 会计算 `0x104f81234 − 0x104f80000 + __TEXT vmaddr` 并解析该结果。如果镜像没有发生滑动，则加载地址等于 `__TEXT` vmaddr（通常是 `0x100000000`），此时运行时地址本身就是静态地址。

### 我从哪里获取加载地址？

从崩溃报告的 **Binary Images** 部分获取。每一行看起来像这样：

```text
0x104f80000 - 0x104fa3fff MyApp arm64 <uuid> /path/MyApp
```

第一列（`0x104f80000`）就是要用 `-l` 传入的加载地址。

## 复现 `atos -offset`

Apple 的 `atos -offset N` 把 `N` 当作相对于镜像 `__TEXT` 基址的偏移。你可以用**默认模式配合零加载地址**得到完全相同的结果：

```bash
# 以下两者等价：
atos  -o sample.dSYM/Contents/Resources/DWARF/sample -offset 0x328
atosl -o sample.dSYM -l 0 0x328
```

两者都会计算 `0x328 + __TEXT vmaddr` 并解析到同一个函数。不需要任何特殊标志。

## `-f` / `--file-offsets` 标志

`-f` 是一个**历史遗留的、向后兼容的**模式。它直接使用 `address − load_address` 作为查找地址，*而不*加上 `__TEXT` vmaddr。

```bash
# 当“值减去 load”等于静态 VM 地址时可解析：
atosl -o sample.dSYM -l 0 -f 0x100000328
```

> **`-f` 与 `atos -offset` 并不相同。** 要匹配 `atos -offset`，请使用如上所示的默认模式配合 `-l 0`。`-f` 保持不变，以便依赖它的现有脚本仍能正常工作。

## 快速决策指南

- 在符号化崩溃报告？使用**默认模式**，配合 Binary Images 中的加载地址。
- 手上是从镜像起始处算起的偏移（类似 `atos -offset`）？使用 **`-l 0 <offset>`**。
- 在维护一个使用了 `-f` 的旧脚本？它的行为与以前完全一致。
