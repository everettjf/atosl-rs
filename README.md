# atosl-rs

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/atosl.svg?style=flat-square&color=EA5312)](https://crates.io/crates/atosl)
[![Docs.rs](https://img.shields.io/docsrs/atosl?style=flat-square&color=2E8555)](https://docs.rs/crate/atosl)
[![GitHub Stars](https://img.shields.io/github/stars/everettjf/atosl-rs?style=flat-square&color=FF6B6B)](https://github.com/everettjf/atosl-rs/stargazers)
[![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.60+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)

A practical Rust CLI for symbolication.  
`atosl` resolves memory addresses to function names and source locations from binaries with symbols/DWARF.

</div>

## Why atosl

Apple's `atos` is handy, but not always available or convenient in cross-platform workflows.  
`atosl` provides a lightweight alternative for address-to-symbol resolution with a simple CLI.

## Features

- Resolve one or more addresses to symbols
- Use Mach-O symbol table and DWARF debug info
- Support file-offset mode (`-f`)
- Select architecture/UUID for fat Mach-O binaries
- Verbose mode for debugging symbolication failures
- Works with Rust-supported platforms (tooling/build environment dependent)

## Installation

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

## Quick Start

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 0x100001234
```

Output shape:

```text
<symbol> (in <binary>) (<file>:<line>)
```

## Usage

```bash
atosl [OPTIONS] -o <OBJECT_PATH> -l <LOAD_ADDRESS> [ADDRESSES]...
```

Required arguments:

- `-o <OBJECT_PATH>`: binary or symbol file path
- `-l <LOAD_ADDRESS>`: load address (`0x...` hex or decimal)
- `[ADDRESSES]...`: one or more addresses to symbolize

Options:

- `-f`: treat addresses as file offsets
- `-v`: verbose diagnostics
- `-a, --arch <ARCH>`: choose architecture in fat Mach-O (`arm64`, `arm64e`, `armv7`, `x86_64`, `i386`, ...)
- `--uuid <UUID>`: choose Mach-O slice by UUID (with or without hyphens)

## Examples

Resolve multiple addresses:

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 0x100001234 0x100005678
```

Use file-offset mode:

```bash
atosl -f -o MyApp.app/MyApp -l 0x0 0x1234
```

Select a fat Mach-O slice by architecture:

```bash
atosl -o Flutter -l 0x100000000 -a arm64 0x100001234
```

Select a fat Mach-O slice by UUID:

```bash
atosl -o Flutter -l 0x100000000 --uuid 34FBD46D-4A1F-3B41-A0F1-4E57D7E25B04 0x100001234
```

## Troubleshooting

- `N/A - no symbols`: binary may be stripped or missing debug sections
- Unexpected source location: verify the load address is correct for the running image
- Wrong symbol on fat binaries: specify `--arch` or `--uuid` explicitly

## Development

```bash
cargo check
cargo build
cargo build --release
cargo fmt
cargo clippy -- -D warnings
```

## Known Limitations

- Not a full 1:1 clone of Apple's `atos`
- Final accuracy depends on symbol and DWARF quality in the input binary

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=everettjf/atosl-rs&type=Date)](https://star-history.com/#everettjf/atosl-rs&Date)

## Contributing

Issues and pull requests are welcome.

## License

MIT. See `LICENSE`.
