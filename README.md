# atosl-rs

🦀️ **atos for linux by rust**

A partial replacement for Apple's `atos` tool for converting addresses within a binary file to symbols.

[![Crates.io](https://img.shields.io/crates/v/atosl.svg)](https://crates.io/crates/atosl)
[![License](https://img.shields.io/crates/l/atosl.svg)](https://github.com/everettjf/atosl-rs/blob/master/LICENSE)

> Tested on DWARF and Mach-O formats.

---

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Examples](#examples)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)
- [Star History](#star-history)

---

## Features

- **Symbolication**: Convert memory addresses to symbols (function names, source files, line numbers).
- **Cross-Platform**: Designed to bring Apple's `atos` functionality to Linux.
- **Format Support**: Supports DWARF and Mach-O binary formats.
- **Performance**: Built with Rust for speed and safety.

## Quick Start

If you have Rust installed, getting started is as simple as:

```bash
cargo install atosl
```

## Installation

### 1. Install Rust
If you don't have Rust installed, follow the official guide:
https://www.rust-lang.org/tools/install

### 2. Install atosl

**Method A: via Cargo (Recommended)**

```bash
# macOS / Linux
cargo install atosl
```

**Method B: Ubuntu / Debian Dependencies**

If you are building on a fresh Ubuntu environment, you might need the following dependencies:

```bash
sudo apt update
sudo apt install git curl build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install atosl
cargo install atosl
```

## Usage

```text
atosl [OPTIONS] -o <OBJECT_PATH> -l <LOAD_ADDRESS> [ADDRESSES]...
```

### Arguments

| Argument | Description |
| :--- | :--- |
| `<ADDRESSES>...` | Addresses to translate (hex or decimal). |

### Options

| Option | Short | Description |
| :--- | :--- | :--- |
| `--object-path` | `-o` | Symbol file path or binary file path. |
| `--load-address` | `-l` | Load address of the binary image. |
| `--file-offset-type` | `-f` | Treat addresses as file offsets (ignore vmaddr in `__TEXT` or executable segments). |
| `--verbose` | `-v` | Enable verbose mode with extra output. |
| `--help` | `-h` | Print help information. |
| `--version` | `-V` | Print version information. |

## Examples

### DWARF Example

```bash
atosl -l 4581015552 -o "full path to dwarf file" 4674962060 4786995348
```

### Mach-O Example

```bash
atosl -l 9093120 -o "full path to libsystem_malloc.dylib" 6754325196
```

## Development

To contribute or run the project locally from source:

1. **Clone the repository:**
   ```bash
   git clone https://github.com/everettjf/atosl-rs.git
   cd atosl-rs
   ```

2. **Run locally:**
   ```bash
   cargo run -- -o <path_to_binary> -l <load_address> <address>
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

## Contributing

Contributions are welcome! If you find a bug or want to optimize the tool, feel free to make a pull request.

Please ensure your code is formatted and linted:
```bash
cargo fmt
cargo clippy
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=everettjf/atosl-rs&type=Date)](https://star-history.com/#everettjf/atosl-rs&Date)