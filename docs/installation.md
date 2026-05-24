---
title: Installation
layout: default
nav_order: 2
---

# Installation

`atosl` is a single self-contained binary. There are two ways to get it.

## From crates.io

```bash
cargo install atosl
```

This builds and installs the `atosl` binary into `~/.cargo/bin`. Make sure that
directory is on your `PATH`.

## From source

```bash
git clone https://github.com/everettjf/atosl-rs.git
cd atosl-rs
cargo build --release
./target/release/atosl --help
```

The release binary is written to `target/release/atosl`.

## Requirements

- A recent stable Rust toolchain (the crate sets `rust-version = "1.85"`).
- No runtime dependency on Xcode or Apple tooling. `atosl` reads Mach-O, DWARF,
  and ELF directly.

## Verify the install

```bash
atosl --version
atosl --help
```

`--help` prints every flag with a short description and is the fastest reference
while you learn the tool.

## Next

Continue to [Getting started](tutorial/getting-started) to run your first
symbolication end to end.
