---
title: 快速上手
layout: default
parent: 中文文档
nav_order: 2
---

# 快速上手

本指南从零开始演示一次完整的符号化：构建一个小程序、生成调试信息，并将一个地址解析为函数和源码行。本指南假设你已经[安装了 atosl](installation)。

## 1. 一次符号化的构成

要将地址转换为符号，`atosl` 需要三样东西：

1. **一个带符号的对象**（`-o`）——可执行文件、目标文件（object file）、dSYM 载荷（payload），或一个 `.dSYM` bundle。
2. **一个加载地址**（`-l`）——镜像被映射到的地址。对于直接来自崩溃报告的值，这就是镜像在 “Binary Images” 部分中的加载地址。
3. **一个或多个待解析的地址**。

```bash
atosl -o <OBJECT> -l <LOAD_ADDRESS> <ADDRESS>...
```

## 2. 构建一个示例（macOS）

```bash
cat > sample.c <<'EOF'
#include <stdio.h>

__attribute__((noinline)) int compute(int n) {
    int acc = 0;
    for (int i = 0; i < n; i++) acc += i * i;
    return acc;
}

int main(int argc, char **argv) {
    printf("%d\n", compute(argc + 5));
    return 0;
}
EOF

# 保留目标文件，以便 dsymutil 收集 DWARF。
clang -g -O1 -arch arm64 -c sample.c -o sample.o
clang -g -arch arm64 sample.o -o sample
dsymutil sample -o sample.dSYM
```

> 在 Linux 上，使用 `gcc -g sample.c -o sample` 编译；DWARF 会被嵌入可执行文件中，因此你可以直接将 `-o` 指向 `sample`。

## 3. 找到一个地址及其加载地址

```bash
# `compute` 的静态 VM 地址：
nm sample | grep compute
# 例如 0000000100000328 T _compute

# 对于未发生滑动（slid）的镜像，其 __TEXT vmaddr 即为自然的加载地址：
otool -l sample | awk '/segname __TEXT/{f=1} f&&/vmaddr/{print; exit}'
# 例如 vmaddr 0x0000000100000000
```

## 4. 符号化

```bash
atosl -o sample.dSYM -l 0x100000000 0x100000328
```

```text
compute (in sample) (sample.c:4)
```

这与 Apple 的 `atos` 给出的答案相同：

```bash
atos -o sample.dSYM/Contents/Resources/DWARF/sample -l 0x100000000 0x100000328
# compute (in sample) (sample.c:4)
```

## 5. 一次解析多个地址

在一次调用中传入多个地址——对象只会被解析一次：

```bash
atosl -o sample.dSYM -l 0x100000000 0x100000328 0x100000360 0x1000003a0
```

每个地址按顺序产生一行输出。

## 6. 读懂输出

当 DWARF 源码信息可用时：

```text
compute (in sample) (sample.c:4)
```

当仅有符号表可用时（例如被裁剪过的二进制）：

```text
compute (in sample) + 16
```

`+ 16` 是相对于匹配符号起始处的字节偏移。

当某个地址无法解析时：

```text
N/A - failed to search symbol table
```

## 接下来

- 不确定该用哪个 `-l` 值，或者你手上是文件偏移而非运行时地址？参见[地址模式](address-modes)。
- 想要供脚本使用的机器可读输出？参见[输出格式](output-formats)。
- 要对包含许多地址的崩溃日志进行符号化？参见[输入来源](input-sources)。
