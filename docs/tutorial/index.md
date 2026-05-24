---
title: Tutorials
layout: default
nav_order: 3
has_children: true
---

# Tutorials

A complete, example-driven tour of `atosl`. The guides are ordered so you can
read them top to bottom, but each one stands on its own.

1. [Getting started](getting-started) — your first end-to-end symbolication
2. [Address modes](address-modes) — load address, file offsets, and the `atos -offset` equivalent
3. [Input sources](input-sources) — dSYM bundles, directories, files, and stdin
4. [Fat binaries & slices](fat-binaries) — selecting an arch/UUID slice
5. [Inline frames](inline-frames) — expanding inlined call stacks
6. [Output formats](output-formats) — text and JSON, with a field reference
7. [Separate debug files](separate-debug-files) — ELF `.gnu_debuglink`, build-id, debuginfod
8. [Library API](library-api) — using atosl as a Rust crate
9. [Troubleshooting](troubleshooting) — errors and known limitations

> Every command on these pages is real and copy-pasteable. Where a command
> compares `atosl` to Apple's `atos`, you need macOS with Xcode command-line
> tools; everything else runs on any platform.
