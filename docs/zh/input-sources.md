---
title: 输入来源
layout: default
parent: 中文文档
nav_order: 4
---

# 输入来源

`atosl` 在*指向什么*以及*地址从哪里来*这两方面都很灵活。本指南将两者都涵盖。

## `-o` 接受什么

`-o/--object` 参数可以是以下任意一种：

| 输入 | 示例 |
| --- | --- |
| 一个可执行文件或目标文件 | `-o MyApp` |
| 一个 dSYM 载荷（bundle 内部的 Mach-O） | `-o MyApp.app.dSYM/Contents/Resources/DWARF/MyApp` |
| 一个 `.dSYM` bundle 目录 | `-o MyApp.app.dSYM` |
| 一个按 UUID/build-id 搜索的目录 | `-o ./symbols --uuid <UUID>` |

### 直接指向 `.dSYM` bundle

你不必深入 bundle 内部。传入该目录，`atosl` 会自动定位其中的 DWARF 载荷：

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 0x100001234
```

### 按 UUID 搜索目录

如果你把许多 dSYM/二进制放在同一个文件夹中，可以让 `atosl` 按 UUID（或 build-id）挑选匹配的那一个：

```bash
atosl -o ./symbols -l 0x100000000 --uuid 34FBD46D4A1F3B41A0F14E57D7E25B04 0x100001234
```

UUID 可以带或不带连字符书写。如果没有任何匹配项，你会得到一个清晰的错误：

```text
no binary or dSYM under ./symbols matched uuid 00000000-0000-0000-0000-000000000000
```

## 地址从哪里来

有三种方式提供地址。它们在优先级上互斥：先是命令行，其次是 `--input`，最后是标准输入（stdin）。

### 1. 在命令行上

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 0x100001234 0x100004321
```

地址可以是十六进制（`0x…`）或十进制。

### 2. 从文件（`--input`）

```bash
printf '0x100001234\n0x100004321\n0x100008888\n' > addrs.txt
atosl -o MyApp.app.dSYM -l 0x100000000 --input addrs.txt
```

地址按行读取，每行可有一个或多个，以空白字符分隔。

### 3. 从标准输入

当你既不提供命令行地址，也不提供 `--input` 时，`atosl` 会读取标准输入。在 `text` 和 `json-lines` 格式下，它会**流式**处理——每读到一个地址就立即打印一条结果，这非常适合管道传入一份很长的崩溃日志：

```bash
grep -o '0x[0-9a-f]\+' crash.txt | atosl -o MyApp.app.dSYM -l 0x100000000
```

```bash
cat addrs.txt | atosl -o MyApp.app.dSYM -l 0x100000000 --format json-lines
```

> 单文档格式（`json`、`json-pretty`）会收集所有结果，并在最后打印一份文档。`text` 和 `json-lines` 则增量式地流式输出。

## 下一步

- 在处理通用（universal）二进制？参见 [Fat 二进制与切片](fat-binaries)。
- 在为你的流水线选择输出形态？参见[输出格式](output-formats)。
