# atosl-rs 🦀️

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/atosl.svg?style=flat-square&color=EA5312)](https://crates.io/crates/atosl)
[![GitHub Stars](https://img.shields.io/github/stars/everettjf/atosl-rs?style=flat-square&color=FF6B6B)](https://github.com/everettjf/atosl-rs/stargazers)
[![GitHub Forks](https://img.shields.io/github/forks/everettjf/atosl-rs?style=flat-square)](https://github.com/everettjf/atosl-rs/network)
[![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.60+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)

**A partial replacement for Apple's `atos` tool**

Convert memory addresses to symbols (function names, source files, line numbers) on Linux and macOS.

[English](README.md)

</div>

> 💡 *Bring Apple's powerful `atos` tool to Linux. Debug iOS/macOS binaries with ease.*

---

## 🎯 What is atosl-rs?

`atosl-rs` is a Rust-based reimplementation of Apple's `atos` (address to symbol) tool. It converts memory addresses within a binary file to human-readable symbol names, source file paths, and line numbers.

### Why Rust?

- ⚡ **Fast** - Native performance
- 🔒 **Safe** - Memory safety guarantees
- 📦 **Portable** - Runs on Linux, macOS, and Windows
- 🛠️ **Easy to build** - Single cargo command

### Supported Formats

| Format | macOS | Linux |
|--------|-------|-------|
| **Mach-O** | ✅ Full | - |
| **DWARF** | ✅ | ✅ |
| **dSYM** | ✅ | - |

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🎯 **Symbolication** | Convert addresses → symbols |
| 📍 **Line Numbers** | Get source file and line info |
| 🔧 **Format Support** | Mach-O, DWARF, dSYM |
| 🖥️ **Cross-Platform** | Linux, macOS, Windows |
| ⚡ **Fast** | Built with Rust |
| 🔄 **CLI Interface** | Simple command-line usage |

---

## 🚀 Quick Start

### Installation

#### Option 1: Cargo Install (Recommended)

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install atosl-rs
cargo install atosl
```

#### Option 2: Build from Source

```bash
git clone https://github.com/everettjf/atosl-rs.git
cd atosl-rs
cargo build --release
```

#### Option 3: Homebrew (macOS)

```bash
brew install atosl-rs  # Coming soon!
```

### Usage

```bash
# Basic symbolication
atosl -o MyApp.app/MyApp 0x100001234

# With line numbers
atosl -l -o MyApp.app/MyApp 0x100001234

# Multiple addresses
atosl -o MyApp.app/MyApp 0x100001234 0x100005678 0x100009ABC

# From stdin
echo "0x100001234" | atosl -o MyApp.app/MyApp
```

---

## 📖 Examples

### Example 1: Basic Symbol Lookup

```bash
$ atosl -o MyApp.app/MyApp 0x100001234

-[MyAppDelegate application:didFinishLaunchingWithOptions:] (in MyApp) (AppDelegate.m:42)
```

### Example 2: Get Line Numbers

```bash
$ atosl -l -o MyApp.app/MyApp 0x100005678

-[MyViewController viewDidLoad] (in MyApp) (ViewController.m:15)
```

### Example 3: Batch Processing

```bash
$ atosl -o MyApp.app/MyApp $(cat addresses.txt)

0x100001234: -[AppDelegate application:didFinishLaunchingWithOptions:] (AppDelegate.m:42)
0x100002345: -[MyViewController viewDidLoad] (ViewController.m:15)
0x100003456: -[NetworkManager fetchDataWithCompletion:] (NetworkManager.m:88)
```

---

## 💻 Integration

### Use as a Library

```rust
use atosl::Symbolicator;

fn main() {
    let symbolicator = Symbolicator::new("path/to/binary").unwrap();
    
    let addresses = vec![0x100001234, 0x100002345];
    let symbols = symbolicator.symbolicate(&addresses).unwrap();
    
    for (addr, symbol) in symbols.iter() {
        println!("{:#x}: {}", addr, symbol);
    }
}
```

### Crate Dependency

```toml
[dependencies]
atosl = "0.1"
```

---

## 🛠️ Development

### Requirements

| Requirement | Version | Description |
|-------------|---------|-------------|
| **Rust** | 1.60+ | Rust toolchain |
| **Cargo** | - | Rust package manager |
| **LLVM/Clang** | - | For DWARF parsing |

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Test

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration

# All tests
cargo test
```

---

## 📊 Performance

```
Symbolication Speed:
- Single address: < 1ms
- 100 addresses: ~5ms
- 1000 addresses: ~50ms

Memory Usage:
- Binary parsing: ~10MB
- Per-symbol: ~100 bytes
```

---

## 🐛 Known Issues

- ❌ 32-bit Mach-O support (limited)
- ⚠️ Some DWARF5 features (in progress)
- 🔄 Symbol order may differ from Apple's atos

---

## 📚 Comparison with Apple atos

| Feature | atos (Apple) | atosl-rs |
|---------|--------------|----------|
| macOS | ✅ Native | ✅ Supported |
| Linux | ❌ | ✅ Supported |
| DWARF | ⚠️ Limited | ✅ Full |
| Speed | Fast | Fast |
| Rust | ❌ | ✅ Native |

---

## 🤝 Contributing

Contributions are welcome!

### How to Contribute

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

### Areas to Help

- 🐛 Bug fixes
- ✨ New binary formats
- 📝 Documentation
- 🧪 Test cases
- ⚡ Performance improvements

---

## 📜 License

atosl-rs is released under the [MIT License](LICENSE).

---

## 🙏 Acknowledgements

Inspired by:
- [Apple's atos](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man1/atos.1.html) - Original tool
- [gaddr](https://github.com/curious-archive/gaddr) - Go implementation
- [llvm-dwarfdump](https://llvm.org/docs/CommandGuide/llvm-dwarfdump.html) - DWARF parsing

---

## 📈 Star History

<div align="center">

[![Star History Chart](https://api.star-history.com/svg?repos=everettjf/atosl-rs&type=Date&theme=dark)](https://star-history.com/#everettjf/atosl-rs&Date)

</div>

---

## 📞 Support

<div align="center">

[![GitHub Issues](https://img.shields.io/badge/Issues-Questions-FF6B6B?style=for-the-badge&logo=github)](https://github.com/everettjf/atosl-rs/issues)
[![Crates.io](https://img.shields.io/badge/Crates-Documentation-EA5312?style=for-the-badge&logo=rust)](https://docs.rs/atosl)
[![GitHub Discussions](https://img.shields.io/badge/Discussions-General-4ECDC4?style=for-the-badge&logo=github)](https://github.com/everettjf/atosl-rs/discussions)

**有问题？去 [Issues](https://github.com/everettjf/atosl-rs/issues) 提问！**

</div>

---

<div align="center">

**Made with ❤️ by [Everett](https://github.com/everettjf)**

**Project Link:** [https://github.com/everettjf/atosl-rs](https://github.com/everettjf/atosl-rs)

**Crate:** [https://crates.io/crates/atosl](https://crates.io/crates/atosl)

</div>
