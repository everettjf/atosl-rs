---
title: Fat binaries & slices
layout: default
parent: Tutorials
nav_order: 4
---

# Fat binaries & slices

A Mach-O **fat** (universal) binary packs multiple architecture slices into one
file. To symbolize, `atosl` needs to know which slice you mean.

## Build a fat binary (macOS)

```bash
clang -g -O1 -arch arm64  -c sample.c -o sample.arm64.o
clang -g -arch arm64  sample.arm64.o  -o sample.arm64
clang -g -O1 -arch x86_64 -c sample.c -o sample.x86.o
clang -g -arch x86_64 sample.x86.o    -o sample.x86

lipo -create sample.arm64 sample.x86 -output fat
lipo -info fat
# Architectures in the fat file: fat are: x86_64 arm64
```

## What happens without a selector

If the binary has more than one slice and you do not choose one, `atosl` refuses
to guess and lists what is available:

```bash
atosl -o fat -l 0x100000000 0x100000460
```

```text
fat Mach-O contains multiple slices.
Use -a/--arch or --uuid to select one.
Available slices:
- arch=x86_64 uuid=80D7C53A-F639-3261-9DC0-4AB2DAF6D0BD
- arch=arm64 uuid=74B3ADB4-0508-3A1E-8B6A-8FC92DACCE66
```

## Select by architecture

```bash
atosl -o fat -l 0x100000000 --arch arm64  0x100000460
atosl -o fat -l 0x100000000 --arch x86_64 0x100000470
```

Common aliases are accepted: `arm64`/`aarch64`, `x86_64`/`amd64`, `i386`/`x86`.

## Select by UUID

When you have the UUID from a crash report, select the slice directly. Hyphens
are optional:

```bash
atosl -o fat -l 0x100000000 --uuid 74B3ADB4-0508-3A1E-8B6A-8FC92DACCE66 0x100000460
atosl -o fat -l 0x100000000 --uuid 80D7C53AF63932619DC04AB2DAF6D0BD       0x100000470
```

## `--arch` / `--uuid` also pick from a directory

The same `--uuid` that selects a slice inside a fat binary also selects a *file*
when `-o` is a directory. See [Input sources](input-sources#search-a-directory-by-uuid).

## Tip: each slice has its own addresses

The same source compiled for two architectures lands at different addresses and
can even map to slightly different source lines after optimization. Always pair
an address with the slice it came from.
