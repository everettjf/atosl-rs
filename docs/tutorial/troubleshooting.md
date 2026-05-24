---
title: Troubleshooting
layout: default
parent: Tutorials
nav_order: 9
---

# Troubleshooting & limitations

## Reading common messages

### `N/A - failed to search symbol table`

The lookup address fell outside every symbol. Usually the address or the load
address is wrong. Re-check [Address modes](address-modes): for a crash report you
want the **default mode** with the image's load address from Binary Images.

### `address 0x… is smaller than load address 0x…`

You subtracted more than the address holds. The value you passed is below the
load address, so it cannot be a runtime address for that image. Either you used
the wrong load address, or the value is a file offset — in that case use
`-l 0 <offset>` (see [Address modes](address-modes#reproducing-atos-offset)).

### `fat Mach-O contains multiple slices`

You pointed at a universal binary without choosing a slice. Add `--arch` or
`--uuid`. See [Fat binaries & slices](fat-binaries).

### `no binary or dSYM under <dir> matched uuid <uuid>`

Directory search found no file with that UUID/build-id. Confirm the UUID (from
the crash report's Binary Images) and that the dSYM for that exact build is in
the directory.

## Known limitations

### Mach-O debug map (`N_OSO`)

When you build with `-g` but **do not** run `dsymutil`, the executable keeps only
`N_OSO` stab entries that point at the original `.o` files; the line-table DWARF
lives in those objects. Apple's `atos` walks that debug map to recover source
lines. `atosl` does **not** follow the debug map, so for such a binary it falls
back to the symbol table (`symbol + offset`).

**Fix:** point `atosl` at a generated `.dSYM` (run `dsymutil`), or at an object
that embeds DWARF directly. Then source locations resolve exactly like `atos`.

### Source paths are printed in full

Apple `atos` prints only the file name unless given `-fullPath`. `atosl` always
prints the path as recorded in the DWARF line table. This is cosmetic; the file
and line are the same.

### Scope

- `atosl` is not a 1:1 clone of `atos`. Mach-O workflows are the primary design
  target; other formats work best when symbols are present.
- Symbolication quality depends on the symbol and DWARF data in the target. No
  tool can recover what is not in the file.
- Real crash-log *ingestion* (parsing a full `.crash`/`.ips` file) is out of
  scope — `atosl` symbolizes addresses; extracting them is up to you.

## Still stuck?

Run with `-v/--verbose` to see resolver diagnostics (which resolver was chosen,
the lookup address per frame) on stderr:

```bash
atosl -v -o app.dSYM -l 0x100000000 --arch arm64 0x100001234
```

If something looks like a bug, open an issue with the command, the `-v` output,
and how it differs from `atos`:
<https://github.com/everettjf/atosl-rs/issues>.
