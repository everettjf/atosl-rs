---
title: 输出格式
layout: default
parent: 中文文档
nav_order: 7
---

# 输出格式

用 `--format` 选择输出形态。默认是 `text`。

| 格式 | 形态 | 能否从 stdin 流式输出？ |
| --- | --- | --- |
| `text` | 每帧一行人类可读的输出 | 是 |
| `json` | 整次运行输出一份 JSON 文档 | 否（收集后输出） |
| `json-pretty` | 同一份文档，带缩进 | 否（收集后输出） |
| `json-lines` | 每个地址一个 JSON 对象（ndjson） | 是 |

## Text

```bash
atosl -o app.dSYM -l 0x100000000 0x100001234
```

```text
my::function (in app) (src/main.rs:42)   # 有 DWARF 源码信息
my::function (in app) + 16               # 符号表回退（偏移）
N/A - failed to search symbol table      # 无法解析
```

关于 `--inline-frames` 如何改变文本输出，参见[内联帧](inline-frames)。

## JSON 文档

`json`（紧凑）和 `json-pretty`（带缩进）会输出单份文档：

```bash
atosl -o app.dSYM -l 0x100000000 --format json-pretty 0x100001234
```

```json
{
  "object_path": "app.dSYM",
  "object_name": "app",
  "selected_slice": {
    "arch": "arm64",
    "uuid": "34FBD46D-4A1F-3B41-A0F1-4E57D7E25B04"
  },
  "frames": [
    {
      "status": "resolved",
      "requested_address": 4294971956,
      "lookup_address": 4660,
      "symbol": "main",
      "object_name": "app",
      "offset": 0,
      "resolver": "symbol_table",
      "location": { "file": "src/main.rs", "line": 12 }
    }
  ]
}
```

## JSON lines（ndjson）

`json-lines` 每个地址打印一个 JSON 对象，每行一个。它在输入模式下流式输出，因此与管道配合得很好：

```bash
printf '0x100001234\n0xdeadbeef\n' | atosl -o app.dSYM -l 0x100000000 --format json-lines
```

```json
{"status":"resolved","requested_address":4294971956,"lookup_address":4660,"symbol":"main","object_name":"app","offset":0,"resolver":"dwarf","location":{"file":"src/main.rs","line":12}}
{"status":"unresolved","requested_address":3735928559,"error":"address is smaller than load address ..."}
```

每一行都是独立且合法的 JSON，因此你可以在结果到达时随即用 `jq` 处理：

```bash
cat addrs.txt | atosl -o app.dSYM -l 0x100000000 --format json-lines \
  | jq -r 'select(.status=="resolved") | "\(.symbol) \(.location.file):\(.location.line)"'
```

## 字段参考

### 报告（json / json-pretty）

| 字段 | 含义 |
| --- | --- |
| `object_path` | 你给定的 `-o` 路径 |
| `object_name` | 镜像的文件名 |
| `selected_slice` | 所选 fat 切片的 `{arch, uuid}`，或为 `null` |
| `frames` | 每个请求地址对应一个结果 |

### 帧结果（也是每个 `json-lines` 行）

一个已解析的帧：

| 字段 | 含义 |
| --- | --- |
| `status` | `"resolved"` |
| `requested_address` | 你传入的地址（十进制） |
| `lookup_address` | 实际查找的静态 VM 地址 |
| `symbol` | 函数名（已还原修饰，demangled） |
| `object_name` | 该符号所属的镜像 |
| `offset` | 相对于符号起始处的字节偏移（符号表结果） |
| `resolver` | `"dwarf"` 或 `"symbol_table"` |
| `location` | 当 DWARF 含有信息时为 `{file, line}`，否则省略 |
| `inlined_by` | 外围的内联帧，最外层排在最后（仅在存在时出现） |

一个未解析的帧：

| 字段 | 含义 |
| --- | --- |
| `status` | `"unresolved"` |
| `requested_address` | 你传入的地址 |
| `error` | 无法解析的原因 |

> `frames` 数组始终与输入地址一一对应，即便存在内联帧也是如此——内联栈位于 `inlined_by` 内部，而不会作为额外的数组项出现。
