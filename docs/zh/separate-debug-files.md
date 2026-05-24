---
title: 独立调试文件
layout: default
parent: 中文文档
nav_order: 8
---

# 独立调试文件（ELF）

在 ELF 平台上，调试信息往往随附在可执行文件*之外*——存放在一个伴随的 `.debug` 文件中。`atosl` 遵循标准机制来查找它。

## 如何定位伴随文件

当主对象缺少 DWARF 时，`atosl` 会按以下顺序查找一个独立的调试文件：

1. **`.gnu_debuglink`**——一个节（section），其中指定了调试文件名以及一个 CRC。CRC 会被校验，因此过期或不匹配的文件会被拒绝，而不是被盲目信任。
2. **Build-id**——使用 `.note.gnu.build-id` 在标准调试根目录下查找 `.build-id/xx/yyyy.debug`。
3. **debuginfod 缓存**——复用本地 debuginfod 缓存目录中已下载的工件（artifact）。

## 它在哪里搜索

默认情况下会查询常见的系统位置。可用 `--debug-dir` 添加更多根目录，该选项可重复使用：

```bash
atosl -o ./myprog -l 0x400000 \
  --debug-dir /usr/lib/debug \
  --debug-dir ./build/debug-symbols \
  0x4011a0
```

每个 `--debug-dir` 都会被用于查找 `.gnu_debuglink` 的目标文件以及 build-id 布局。

## CRC 很重要

`.gnu_debuglink` 携带了预期调试文件的 CRC。如果你手上的文件不匹配，`atosl` 会拒绝使用它：

```text
.gnu_debuglink target found but CRC did not match; ignoring stale debug file
```

这能保护你不会悄无声息地用错误的构建去做符号化。

## debuginfod

如果你已经在使用 `debuginfod` 并且它的客户端已经填充了本地缓存，`atosl` 会复用那些文件。`atosl` 自身不会执行网络下载；它只读取已缓存的内容。

## 关于 Mach-O 的说明

本章针对 ELF。对于 Apple 平台，独立调试文件的等价物是 **`.dSYM` bundle**——直接将 `-o` 指向它即可，如[输入来源](input-sources)所述。另请参阅[排错与限制](troubleshooting)中关于调试映射（debug map）的限制说明。
