---
title: Getting started
layout: default
parent: Tutorials
nav_order: 1
---

# Getting started

This guide walks through a complete symbolication from scratch: build a small
program, produce debug info, and resolve an address to a function and source
line. It assumes you have [installed atosl](../installation).

## 1. The anatomy of a symbolication

To turn an address into a symbol, `atosl` needs three things:

1. **An object with symbols** (`-o`) — an executable, an object file, a dSYM
   payload, or a `.dSYM` bundle.
2. **A load address** (`-l`) — the address the image was mapped at. For a value
   straight out of a crash report, this is the image's load address from the
   "Binary Images" section.
3. **One or more addresses** to resolve.

```bash
atosl -o <OBJECT> -l <LOAD_ADDRESS> <ADDRESS>...
```

## 2. Build a sample (macOS)

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

# Keep the object file so dsymutil can collect DWARF.
clang -g -O1 -arch arm64 -c sample.c -o sample.o
clang -g -arch arm64 sample.o -o sample
dsymutil sample -o sample.dSYM
```

> On Linux, compile with `gcc -g sample.c -o sample`; the DWARF is embedded in
> the executable, so you can point `-o` straight at `sample`.

## 3. Find an address and its load address

```bash
# The static VM address of `compute`:
nm sample | grep compute
# e.g. 0000000100000328 T _compute

# The image's __TEXT vmaddr is the natural load address for a non-slid image:
otool -l sample | awk '/segname __TEXT/{f=1} f&&/vmaddr/{print; exit}'
# e.g. vmaddr 0x0000000100000000
```

## 4. Symbolize

```bash
atosl -o sample.dSYM -l 0x100000000 0x100000328
```

```text
compute (in sample) (sample.c:4)
```

That is the same answer Apple's `atos` gives:

```bash
atos -o sample.dSYM/Contents/Resources/DWARF/sample -l 0x100000000 0x100000328
# compute (in sample) (sample.c:4)
```

## 5. Resolve several addresses at once

Pass multiple addresses in one invocation — the object is parsed only once:

```bash
atosl -o sample.dSYM -l 0x100000000 0x100000328 0x100000360 0x1000003a0
```

Each address produces one line of output, in order.

## 6. Read the output

When DWARF source info is available:

```text
compute (in sample) (sample.c:4)
```

When only the symbol table is available (for example a stripped binary):

```text
compute (in sample) + 16
```

The `+ 16` is the byte offset from the start of the matched symbol.

When an address cannot be resolved:

```text
N/A - failed to search symbol table
```

## What's next

- Not sure which `-l` value to use, or have a file offset instead of a runtime
  address? See [Address modes](address-modes).
- Want machine-readable output for a script? See [Output formats](output-formats).
- Symbolizing a crash log with many addresses? See [Input sources](input-sources).
