---
title: Home
layout: default
nav_order: 1
---

# atosl
{: .fs-9 }

A fast, local symbolication CLI and Rust library — an `atos`-style tool that turns
raw binary addresses into function names and source locations.
{: .fs-6 .fw-300 }

[Get started](tutorial/getting-started){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[View on GitHub](https://github.com/everettjf/atosl-rs){: .btn .fs-5 .mb-4 .mb-md-0 }
[中文文档](zh/){: .btn .fs-5 .mb-4 .mb-md-0 }

---

## What is atosl?

`atosl` resolves addresses to symbols using **DWARF** debug info when it is
available, and falls back to the **symbol table** when it is not. It is built for
cross-platform tooling, CI pipelines, and crash-processing utilities that need
`atos`-style symbolication without depending on Apple's host environment.

Unlike Apple's `atos`, it runs anywhere, ships as a single binary, and exposes a
small embeddable Rust API.

## 30-second quickstart

```bash
# Install
cargo install atosl

# Symbolize one address against a dSYM bundle
atosl -o MyApp.app.dSYM -l 0x100000000 0x100001234

# Symbolize several addresses, machine-readable
atosl -o MyApp.app.dSYM -l 0x100000000 --format json 0x100001234 0x100004321

# Pipe a list of addresses through, one ndjson result per line
printf '0x100001234\n0x100004321\n' | atosl -o MyApp.app.dSYM -l 0x100000000 --format json-lines
```

## What it handles

- Executables, object files, dSYM payloads, and `.dSYM` bundle directories
- A directory of symbols searched by `--uuid` or build-id
- Mach-O fat binaries with explicit slice selection (`--arch`, `--uuid`)
- Inline call-stack expansion (`--inline-frames`, like `atos -i`)
- Addresses from the command line, a file (`--input`), or stdin
- Output as `text`, `json`, `json-pretty`, or streaming `json-lines`
- Separate ELF debug files via `.gnu_debuglink`, build-id, or the debuginfod cache

## Where to go next

| Guide | What it covers |
| --- | --- |
| [Installation](installation) | Installing from crates.io or building from source |
| [Getting started](tutorial/getting-started) | Your first end-to-end symbolication |
| [Address modes](tutorial/address-modes) | Load address vs. file offsets, and the `atos -offset` equivalent |
| [Input sources](tutorial/input-sources) | dSYM bundles, directories, files, and stdin |
| [Fat binaries & slices](tutorial/fat-binaries) | Selecting an arch/UUID slice from a universal binary |
| [Inline frames](tutorial/inline-frames) | Expanding inlined call stacks |
| [Output formats](tutorial/output-formats) | Text and JSON shapes, with a field reference |
| [Separate debug files](tutorial/separate-debug-files) | ELF `.gnu_debuglink`, build-id, debuginfod |
| [Library API](tutorial/library-api) | Using atosl as a Rust crate |
| [Troubleshooting](tutorial/troubleshooting) | Reading errors and known limitations |

> A full Chinese version of these guides is available under [中文文档](zh/).
