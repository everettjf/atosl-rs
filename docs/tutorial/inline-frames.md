---
title: Inline frames
layout: default
parent: Tutorials
nav_order: 5
---

# Inline frames

When the compiler inlines a function, a single address can correspond to several
nested source locations. `atosl` can report just the containing function or the
whole inline call stack.

## Default: outermost frame only

By default, text output prints only the **outermost** frame — the real,
non-inlined function that physically contains the address. This matches plain
Apple `atos` and the output of earlier `atosl` releases:

```bash
atosl -o app.dSYM -l 0x100000000 0x100000460
```

```text
outer (in app) (outer.c:15)
```

## `--inline-frames`: the full stack

Pass `--inline-frames` to expand the inline call stack, innermost frame first —
the same as `atos -i` / `atos --inlineFrames`:

```bash
atosl -o app.dSYM -l 0x100000000 --inline-frames 0x100000460
```

```text
leaf_inline (in app) (helpers.c:5)
mid_inline (in app) (helpers.c:10)
outer (in app) (outer.c:15)
```

Read it top to bottom as "innermost → outermost": the address is inside
`leaf_inline`, which was inlined into `mid_inline`, which was inlined into
`outer`.

## JSON always carries the full chain

The flag only affects **text** output. JSON output is unchanged regardless of
the flag: it always reports the innermost frame as the primary result and lists
the enclosing inline frames under `inlined_by`.

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

So machine-readable consumers always have the complete information and can choose
how to display it; the `--inline-frames` flag is purely a text-rendering choice.

## When do inline frames appear?

Only when the DWARF actually contains inline records, which generally requires
optimization (`-O1`/`-O2`) plus `always_inline` or compiler-chosen inlining. At
`-O0`, helper calls are usually real calls and there is nothing to expand.
