---
title: Fat 二进制与切片
layout: default
parent: 中文文档
nav_order: 5
---

# Fat 二进制与切片

Mach-O **fat**（通用，universal）二进制将多个架构切片（slice）打包到一个文件中。要进行符号化，`atosl` 需要知道你指的是哪个切片。

## 构建一个 fat 二进制（macOS）

```bash
clang -g -O1 -arch arm64  -c sample.c -o sample.arm64.o
clang -g -arch arm64  sample.arm64.o  -o sample.arm64
clang -g -O1 -arch x86_64 -c sample.c -o sample.x86.o
clang -g -arch x86_64 sample.x86.o    -o sample.x86

lipo -create sample.arm64 sample.x86 -output fat
lipo -info fat
# Architectures in the fat file: fat are: x86_64 arm64
```

## 不指定选择器时会发生什么

如果二进制包含多个切片而你没有选择其中一个，`atosl` 会拒绝猜测，并列出可用的切片：

```bash
atosl -o fat -l 0x100000000 0x100000460
```

```text
fat Mach-O contains multiple slices.
Use -a/--arch or --uuid to select one.
Available slices:
- arch=x86_64 uuid=80D7C53A-F639-3261-9DC0-4AB2DAF6D0BD
- arch=arm64 uuid=74B3ADB4-0508-3A1E-8B6A-8FC92DACCE66
```

## 按架构选择

```bash
atosl -o fat -l 0x100000000 --arch arm64  0x100000460
atosl -o fat -l 0x100000000 --arch x86_64 0x100000470
```

可接受常见别名：`arm64`/`aarch64`、`x86_64`/`amd64`、`i386`/`x86`。

## 按 UUID 选择

当你从崩溃报告中获得了 UUID 时，可以直接选择对应切片。连字符是可选的：

```bash
atosl -o fat -l 0x100000000 --uuid 74B3ADB4-0508-3A1E-8B6A-8FC92DACCE66 0x100000460
atosl -o fat -l 0x100000000 --uuid 80D7C53AF63932619DC04AB2DAF6D0BD       0x100000470
```

## `--arch` / `--uuid` 也可从目录中挑选

用于在 fat 二进制内部选择切片的同一个 `--uuid`，当 `-o` 是一个目录时，也可用于选择一个*文件*。参见[输入来源](input-sources)。

## 提示：每个切片都有自己的地址

同一份源码为两种架构编译后，会落在不同的地址上，经过优化后甚至可能映射到略有差异的源码行。请始终将地址与它所来自的切片配对使用。
