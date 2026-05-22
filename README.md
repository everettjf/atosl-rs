# atosl-rs

`atosl` is a Rust CLI and library for local symbolication. It resolves raw binary addresses into function names and source locations using DWARF when available and falls back to symbol tables when debug info is missing.

It is designed for cross-platform tooling, CI pipelines, crash-processing utilities, and developer workflows that need `atos`-style symbolication without depending on Apple's host environment.

## Why this exists

Apple's `atos` is useful, but it is tightly coupled to Apple's runtime environment. `atosl` focuses on the parts teams usually need in build systems and tooling:

- A single local binary and embeddable Rust API
- Script-friendly output in `text`, `json`, and `json-pretty`
- DWARF-first resolution with symbol-table fallback
- Fat Mach-O slice selection by architecture or UUID
- Reproducible regression coverage for Apple-specific behavior

## Current quality bar

- `clap` v4 CLI with explicit long-form flags
- Structured JSON and pretty JSON output
- Unit tests for parsing, UUID handling, architecture aliases, and address math
- Integration tests that build a real fixture binary and validate end-to-end symbolization
- Apple-specific Mach-O/DWARF golden tests with reproducible fixtures and checked-in snapshots
- Fat Mach-O slice-selection goldens for `--arch` and `--uuid`
- JSON output goldens for Apple single-slice and fat-binary workflows
- Verbose diagnostic goldens for resolver selection and per-frame lookup tracing
- Criterion benchmark target for batch symbolication throughput
- GitHub Actions CI for `fmt`, `clippy`, tests, and release builds

## What it handles well

- Local symbolication from executables, object files, and dSYM payloads
- Inlined call-stack expansion for DWARF frames, like `atos`
- Multi-address lookups in a single invocation
- Addresses from the command line, a file (`--input`), or stdin (streamed in text mode)
- `.dSYM` bundle directories, or a directory searched by `--uuid` / build-id
- Separate ELF debug files via `.gnu_debuglink` or build-id
- Mach-O fat binaries with explicit slice selection
- Machine-readable integration through JSON output
- Debugging symbolication decisions through verbose diagnostics

## Installation

From crates.io:

```bash
cargo install atosl
```

From source:

```bash
git clone https://github.com/everettjf/atosl-rs.git
cd atosl-rs
cargo build --release
./target/release/atosl --help
```

## Usage

```bash
atosl -o <OBJECT_PATH> -l <LOAD_ADDRESS> [OPTIONS] <ADDRESS>...
```

Required arguments:

- `-o, --object <OBJECT_PATH>`: object file, executable, dSYM payload, `.dSYM` bundle directory, or a directory to search with `--uuid`
- `-l, --load-address <LOAD_ADDRESS>`: runtime image load address
- `<ADDRESS>...`: addresses to symbolize; omit to read from `--input` or stdin

Key options:

- `-f, --file-offsets`: interpret addresses as file offsets
- `-a, --arch <ARCH>`: choose a Mach-O slice in a fat binary
- `--uuid <UUID>`: choose a Mach-O slice by UUID, or select a file from a directory by UUID/build-id
- `-i, --input <FILE>`: read addresses from a file (defaults to stdin when no addresses are given)
- `--format <text|json|json-pretty>`: select output format
- `-v, --verbose`: print resolver diagnostics to stderr

## Examples

Symbolize a single address:

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 0x100001234
```

Symbolize multiple addresses:

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 0x100001234 0x100004321 0x100008888
```

Point directly at a `.dSYM` bundle (the DWARF payload is located automatically):

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 0x100001234
```

Select a specific fat Mach-O slice:

```bash
atosl -o Flutter -l 0x100000000 --arch arm64 0x100001234
```

Read addresses from stdin (text output streams one result per line):

```bash
printf '0x100001234\n0x100004321\n' | atosl -o MyApp.app.dSYM -l 0x100000000
```

Read addresses from a file:

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 --input crash_addresses.txt
```

Search a directory of dSYMs/binaries for the matching image by UUID (or build-id):

```bash
atosl -o ./symbols -l 0x100000000 --uuid 34FBD46D4A1F3B41A0F14E57D7E25B04 0x100001234
```

Emit machine-readable output:

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 --format json 0x100001234
```

Use verbose diagnostics to inspect resolver behavior:

```bash
atosl -v -o MyApp.app/MyApp -l 0x100000000 --arch arm64 0x100001234
```

Example JSON shape:

```json
{
  "object_path": "MyApp.app/MyApp",
  "object_name": "MyApp",
  "selected_slice": {
    "arch": "arm64",
    "uuid": "34FBD46D-4A1F-3B41-A0F1-4E57D7E25B04"
  },
  "frames": [
    {
      "status": "resolved",
      "requested_address": 4294971956,
      "lookup_address": 4660,
      "symbol": "main",
      "object_name": "MyApp",
      "offset": 0,
      "resolver": "symbol_table",
      "location": {
        "file": "src/main.rs",
        "line": 12
      }
    }
  ]
}
```

## Text output

When DWARF source information is available:

```text
my::function (in MyApp) (src/main.rs:42)
```

When only the symbol table is available:

```text
my::function (in MyApp) + 16
```

When symbolication fails:

```text
N/A - failed to search symbol table
```

## Library usage

`atosl` now exposes a library API as well as the CLI:

```rust
use atosl::{atosl, OutputFormat, SymbolizeOptions};

let report = atosl::symbolize_path(&SymbolizeOptions {
    object_path: "fixture_bin".into(),
    load_address: 0,
    addresses: vec![0x1234],
    verbose: false,
    file_offsets: false,
    arch: None,
    uuid: None,
    format: OutputFormat::Json,
})?;
```

The returned `SymbolizeReport` preserves the selected slice, per-address resolver choice, lookup address, symbol name, and optional source location.

## Regression assets

Apple-specific behavior is protected by checked-in goldens under `tests/golden/apple/`:

- Text output for DWARF-backed and symbol-table-backed Mach-O inputs
- JSON output for single-slice and fat Mach-O workflows
- Verbose diagnostics for resolver tracing and slice selection
- Negative-path coverage for ambiguous fat binaries

Refresh those snapshots on macOS with:

```bash
./scripts/refresh_apple_goldens.sh
```

## Development

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
./scripts/refresh_apple_goldens.sh
cargo bench --bench batch_symbolize
```

Run the benchmark binary without executing it in CI-style validation:

```bash
cargo bench --bench batch_symbolize --no-run
```

Release steps are documented in [RELEASING.md](/Users/eevv/focus/atosl-rs/RELEASING.md).
For a one-command release flow, run `./deploy.sh [patch|minor|major|X.Y.Z]`.

## Known limitations

- This is still not a 1:1 clone of Apple's `atos`
- Symbolication quality depends on the symbol and DWARF data in the target binary
- Mach-O workflows remain the primary design target; other object formats work best when symbols are present
- Apple UUIDs and dSYM layouts are covered in tests, but real crash-log ingestion is still out of scope

## License

MIT. See `LICENSE`.
