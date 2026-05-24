---
title: Address modes
layout: default
parent: Tutorials
nav_order: 2
---

# Address modes

The single most common source of confusion in symbolication is *what an address
means*. `atosl` interprets each input address according to the flags you pass.

## The lookup address

Internally, every symbolication resolves a **static link-time VM address** — the
address as recorded in the DWARF line table and symbol table (for a typical
arm64 executable these start at `0x100000000`). The flags decide how your input
is converted to that lookup address.

| Mode | Flags | Lookup address | Typical use |
| --- | --- | --- | --- |
| Load-address (default) | `-l <load>` | `address − load_address + __TEXT vmaddr` | Runtime/virtual addresses from a crash report |
| `atos -offset` equivalent | `-l 0 <off>` | `off + __TEXT vmaddr` | A file offset from the image's `__TEXT` base |
| File offsets (legacy `-f`) | `-f -l <load>` | `address − load_address` | Backward-compatible mode that skips `__TEXT` re-basing |

## Default mode: runtime addresses

This is what you use for crash reports. You pass the **load address** the image
was mapped at and the **runtime addresses** the crash recorded.

```bash
atosl -o MyApp.app.dSYM -l 0x104f80000 0x104f81234
```

`atosl` computes `0x104f81234 − 0x104f80000 + __TEXT vmaddr` and resolves that.
If the image was not slid, the load address equals the `__TEXT` vmaddr (often
`0x100000000`), and the runtime address is already the static address.

### Where do I get the load address?

From the crash report's **Binary Images** section. Each line looks like:

```text
0x104f80000 - 0x104fa3fff MyApp arm64 <uuid> /path/MyApp
```

The first column (`0x104f80000`) is the load address to pass with `-l`.

## Reproducing `atos -offset`

Apple's `atos -offset N` treats `N` as an offset from the image's `__TEXT` base.
You get the identical result from the **default mode with a zero load address**:

```bash
# These two are equivalent:
atos  -o sample.dSYM/Contents/Resources/DWARF/sample -offset 0x328
atosl -o sample.dSYM -l 0 0x328
```

Both compute `0x328 + __TEXT vmaddr` and resolve the same function. No special
flag is needed.

## The `-f` / `--file-offsets` flag

`-f` is a **historical, backward-compatible** mode. It uses
`address − load_address` directly as the lookup address, *without* adding the
`__TEXT` vmaddr.

```bash
# Resolves when the value minus load equals the static VM address:
atosl -o sample.dSYM -l 0 -f 0x100000328
```

> **`-f` is not the same as `atos -offset`.** To match `atos -offset`, use the
> default mode with `-l 0` as shown above. `-f` is kept unchanged so existing
> scripts that relied on it keep working.

## Quick decision guide

- Symbolizing a crash report? Use the **default mode** with the load address
  from Binary Images.
- Have an offset from the start of the image (like `atos -offset`)? Use
  **`-l 0 <offset>`**.
- Maintaining an old script that used `-f`? It still behaves exactly as before.
