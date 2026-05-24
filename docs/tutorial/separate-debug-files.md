---
title: Separate debug files
layout: default
parent: Tutorials
nav_order: 7
---

# Separate debug files (ELF)

On ELF platforms, debug info is often shipped *outside* the executable — in a
companion `.debug` file. `atosl` follows the standard mechanisms to find it.

## How the companion is located

When the main object lacks DWARF, `atosl` looks for a separate debug file using,
in order:

1. **`.gnu_debuglink`** — a section naming a debug file plus a CRC. The CRC is
   verified, so a stale or mismatched file is rejected rather than trusted.
2. **Build-id** — the `.note.gnu.build-id` is used to find
   `.build-id/xx/yyyy.debug` under the standard debug roots.
3. **The debuginfod cache** — already-downloaded artifacts in the local
   debuginfod cache directory are reused.

## Where it searches

By default the usual system locations are consulted. Add more roots with
`--debug-dir`, which is repeatable:

```bash
atosl -o ./myprog -l 0x400000 \
  --debug-dir /usr/lib/debug \
  --debug-dir ./build/debug-symbols \
  0x4011a0
```

Each `--debug-dir` is searched for both the `.gnu_debuglink` target and the
build-id layout.

## CRC matters

`.gnu_debuglink` carries a CRC of the intended debug file. If the file you have
does not match, `atosl` declines to use it:

```text
.gnu_debuglink target found but CRC did not match; ignoring stale debug file
```

This protects you from silently symbolizing against the wrong build.

## debuginfod

If you already use `debuginfod` and its client has populated the local cache,
`atosl` will reuse those files. `atosl` itself does not perform network
downloads; it reads what is already cached.

## Mach-O note

This chapter is about ELF. For Apple platforms the equivalent of a separate debug
file is the **`.dSYM` bundle** — point `-o` at it directly, as described in
[Input sources](input-sources). See also the debug-map limitation in
[Troubleshooting](troubleshooting#mach-o-debug-map-n_oso).
