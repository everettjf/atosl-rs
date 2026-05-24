---
title: 内联帧
layout: default
parent: 中文文档
nav_order: 6
---

# 内联帧

当编译器内联（inline）一个函数时，单个地址可能对应到多个嵌套的源码位置。`atosl` 既可以只报告包含该地址的那个函数，也可以报告完整的内联调用栈。

## 默认：仅最外层帧

默认情况下，文本输出只打印**最外层**的帧——即物理上包含该地址的、真正未被内联的函数。这与原生 Apple `atos` 以及早期 `atosl` 版本的输出一致：

```bash
atosl -o app.dSYM -l 0x100000000 0x100000460
```

```text
outer (in app) (outer.c:15)
```

## `--inline-frames`：完整栈

传入 `--inline-frames` 以展开内联调用栈，最内层的帧排在最前——这与 `atos -i` / `atos --inlineFrames` 相同：

```bash
atosl -o app.dSYM -l 0x100000000 --inline-frames 0x100000460
```

```text
leaf_inline (in app) (helpers.c:5)
mid_inline (in app) (helpers.c:10)
outer (in app) (outer.c:15)
```

从上到下按 “最内层 → 最外层” 来阅读：该地址位于 `leaf_inline` 内部，`leaf_inline` 被内联进 `mid_inline`，而 `mid_inline` 又被内联进 `outer`。

## JSON 始终携带完整链条

该标志只影响**文本**输出。无论标志是否设置，JSON 输出都不变：它始终将最内层的帧作为主结果报告，并将外围的内联帧列在 `inlined_by` 之下。

```bash
atosl -o app.dSYM -l 0x100000000 --format json-pretty 0x100000460
```

```json
{
  "status": "resolved",
  "symbol": "leaf_inline",
  "resolver": "dwarf",
  "location": { "file": "helpers.c", "line": 5 },
  "inlined_by": [
    { "symbol": "mid_inline", "location": { "file": "helpers.c", "line": 10 } },
    { "symbol": "outer",      "location": { "file": "outer.c",   "line": 15 } }
  ]
}
```

因此，机器可读的消费方始终拥有完整的信息，可以自行决定如何展示；`--inline-frames` 标志纯粹是一个文本渲染的选择。

## 内联帧何时出现？

只有当 DWARF 中确实包含内联记录时才会出现，这通常需要开启优化（`-O1`/`-O2`），再加上 `always_inline` 或编译器自行决定的内联。在 `-O0` 下，辅助函数调用通常是真实的调用，没有可展开的内容。
