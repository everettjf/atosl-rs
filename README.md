# atosl-rs

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/atosl.svg?style=flat-square&color=EA5312)](https://crates.io/crates/atosl)
[![GitHub Stars](https://img.shields.io/github/stars/everettjf/atosl-rs?style=flat-square&color=FF6B6B)](https://github.com/everettjf/atosl-rs/stargazers)
[![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.60+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)

A partial replacement for Apple's `atos`.

Convert memory addresses to symbols (function names, source files, line numbers) from binaries with symbol info.

</div>

## Overview

`atosl-rs` is a Rust CLI tool that resolves addresses to readable symbols.

- Works with Mach-O symbol table and DWARF debug info
- Supports verbose output for troubleshooting
- Supports file-offset mode for raw offsets
- Runs on macOS, Linux, and Windows (Rust target support)

## Install

### From crates.io

```bash
cargo install atosl
```

### From source

```bash
git clone https://github.com/everettjf/atosl-rs.git
cd atosl-rs
cargo build --release
```

Binary path:

```bash
./target/release/atosl
```

## Usage

CLI help:

```bash
atosl --help
```

Current syntax:

```bash
atosl [OPTIONS] -o <OBJECT_PATH> -l <LOAD_ADDRESS> [ADDRESSES]...
```

Required arguments:

- `-o <OBJECT_PATH>`: binary/symbol file path
- `-l <LOAD_ADDRESS>`: load address (hex like `0x100000000` or decimal)
- `[ADDRESSES]...`: one or more addresses to symbolize

Options:

- `-f`: treat input addresses as file offsets
- `-v`: verbose debug output
- `-a, --arch <ARCH>`: select architecture for Mach-O fat files (for example `arm64`, `arm64e`, `armv7`, `x86_64`, `i386`)
- `--uuid <UUID>`: select Mach-O slice by UUID (with or without `-`)

### Examples

Resolve one address:

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 0x100001234
```

Resolve multiple addresses:

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 0x100001234 0x100005678
```

Use file-offset mode:

```bash
atosl -f -o MyApp.app/MyApp -l 0x0 0x1234
```

Enable verbose output:

```bash
atosl -v -o MyApp.app/MyApp -l 0x100000000 0x100001234
```

Select slice from a fat Mach-O by architecture:

```bash
atosl -o Flutter -l 0x100000000 -a arm64 0x100001234
```

Select slice from a fat Mach-O by UUID:

```bash
atosl -o Flutter -l 0x100000000 --uuid 34FBD46D-4A1F-3B41-A0F1-4E57D7E25B04 0x100001234
```

## Output

Typical output format:

```text
<symbol> (in <binary>) (<file>:<line>)
```

If only symbol table match is available:

```text
<symbol> (in <binary>) + <offset>
```

If not found:

```text
N/A - <reason>
```

## Development

```bash
cargo check
cargo build
cargo build --release
cargo fmt
cargo clippy -- -D warnings
```

## Known limitations

- Not a 1:1 feature-complete clone of Apple's `atos`
- Accuracy depends on available symbols/DWARF sections in the input binary

## Contributing

Issues and pull requests are welcome.

## License

MIT. See `LICENSE`.
