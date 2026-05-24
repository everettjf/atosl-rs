---
title: Output formats
layout: default
parent: Tutorials
nav_order: 6
---

# Output formats

Choose an output shape with `--format`. The default is `text`.

| Format | Shape | Streams from stdin? |
| --- | --- | --- |
| `text` | One human-readable line per frame | Yes |
| `json` | One JSON document for the whole run | No (collected) |
| `json-pretty` | The same document, indented | No (collected) |
| `json-lines` | One JSON object per address (ndjson) | Yes |

## Text

```bash
atosl -o app.dSYM -l 0x100000000 0x100001234
```

```text
my::function (in app) (src/main.rs:42)   # DWARF source available
my::function (in app) + 16               # symbol-table fallback (offset)
N/A - failed to search symbol table      # could not resolve
```

See [Inline frames](inline-frames) for how `--inline-frames` changes text output.

## JSON document

`json` (compact) and `json-pretty` (indented) emit a single document:

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

## JSON lines (ndjson)

`json-lines` prints one JSON object per address, one per line. It streams in
input mode, so it pairs well with a pipe:

```bash
printf '0x100001234\n0xdeadbeef\n' | atosl -o app.dSYM -l 0x100000000 --format json-lines
```

```json
{"status":"resolved","requested_address":4294971956,"lookup_address":4660,"symbol":"main","object_name":"app","offset":0,"resolver":"dwarf","location":{"file":"src/main.rs","line":12}}
{"status":"unresolved","requested_address":3735928559,"error":"address is smaller than load address ..."}
```

Each line is independently valid JSON, so you can process it with `jq` as it
arrives:

```bash
cat addrs.txt | atosl -o app.dSYM -l 0x100000000 --format json-lines \
  | jq -r 'select(.status=="resolved") | "\(.symbol) \(.location.file):\(.location.line)"'
```

## Field reference

### Report (json / json-pretty)

| Field | Meaning |
| --- | --- |
| `object_path` | The `-o` path as given |
| `object_name` | The image's file name |
| `selected_slice` | `{arch, uuid}` of the chosen fat slice, or `null` |
| `frames` | One outcome per requested address |

### Frame outcome (also each `json-lines` row)

A resolved frame:

| Field | Meaning |
| --- | --- |
| `status` | `"resolved"` |
| `requested_address` | The address you passed (decimal) |
| `lookup_address` | The static VM address actually looked up |
| `symbol` | Function name (demangled) |
| `object_name` | Image the symbol belongs to |
| `offset` | Byte offset from the symbol start (symbol-table results) |
| `resolver` | `"dwarf"` or `"symbol_table"` |
| `location` | `{file, line}` when DWARF has it, else omitted |
| `inlined_by` | Enclosing inline frames, outermost last (only when present) |

An unresolved frame:

| Field | Meaning |
| --- | --- |
| `status` | `"unresolved"` |
| `requested_address` | The address you passed |
| `error` | Why it could not be resolved |

> The `frames` array is always 1:1 with the input addresses, even when inline
> frames are present — the inline stack lives inside `inlined_by`, not as extra
> array entries.
