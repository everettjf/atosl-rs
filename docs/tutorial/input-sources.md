---
title: Input sources
layout: default
parent: Tutorials
nav_order: 3
---

# Input sources

`atosl` is flexible about *what* you point it at and *where* the addresses come
from. This guide covers both.

## What `-o` accepts

The `-o/--object` argument can be any of:

| Input | Example |
| --- | --- |
| An executable or object file | `-o MyApp` |
| A dSYM payload (the Mach-O inside the bundle) | `-o MyApp.app.dSYM/Contents/Resources/DWARF/MyApp` |
| A `.dSYM` bundle directory | `-o MyApp.app.dSYM` |
| A directory to search by UUID/build-id | `-o ./symbols --uuid <UUID>` |

### Point straight at a `.dSYM` bundle

You do not have to dig into the bundle. Pass the directory and `atosl` locates
the DWARF payload inside it automatically:

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 0x100001234
```

### Search a directory by UUID

If you keep many dSYMs/binaries in one folder, let `atosl` pick the matching one
by UUID (or build-id):

```bash
atosl -o ./symbols -l 0x100000000 --uuid 34FBD46D4A1F3B41A0F14E57D7E25B04 0x100001234
```

The UUID can be written with or without hyphens. If nothing matches you get a
clear error:

```text
no binary or dSYM under ./symbols matched uuid 00000000-0000-0000-0000-000000000000
```

## Where addresses come from

There are three ways to feed addresses. They are mutually exclusive in
precedence: command-line first, then `--input`, then stdin.

### 1. On the command line

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 0x100001234 0x100004321
```

Addresses may be hex (`0x…`) or decimal.

### 2. From a file (`--input`)

```bash
printf '0x100001234\n0x100004321\n0x100008888\n' > addrs.txt
atosl -o MyApp.app.dSYM -l 0x100000000 --input addrs.txt
```

Addresses are read one or more per line, whitespace-separated.

### 3. From stdin

When you give neither command-line addresses nor `--input`, `atosl` reads stdin.
In `text` and `json-lines` formats it **streams** — one result is printed as soon
as each address is read, which is ideal for piping a long crash log:

```bash
grep -o '0x[0-9a-f]\+' crash.txt | atosl -o MyApp.app.dSYM -l 0x100000000
```

```bash
cat addrs.txt | atosl -o MyApp.app.dSYM -l 0x100000000 --format json-lines
```

> The single-document formats (`json`, `json-pretty`) collect all results and
> print one document at the end. `text` and `json-lines` stream incrementally.

## Next

- Working with a universal binary? See [Fat binaries & slices](fat-binaries).
- Choosing an output shape for your pipeline? See [Output formats](output-formats).
