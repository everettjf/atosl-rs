# atosl-rs

*Read this in other languages: [简体中文](README.zh-CN.md).*

`atosl` is a Rust CLI and library for local symbolication. It resolves raw binary addresses into function names and source locations using DWARF when available and falls back to symbol tables when debug info is missing.

It is designed for cross-platform tooling, CI pipelines, crash-processing utilities, and developer workflows that need `atos`-style symbolication without depending on Apple's host environment.

## Why this exists

Apple's `atos` is useful, but it is tightly coupled to Apple's runtime environment. `atosl` focuses on the parts teams usually need in build systems and tooling:

- A single local binary and embeddable Rust API
- Script-friendly output in `text`, `json`, `json-pretty`, and streaming `json-lines`
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
- Differential tests that compare `atosl` against Apple's own `/usr/bin/atos` (DWARF, inline frames, and `-f`/`-offset` mode) on macOS
- Criterion benchmark target for batch symbolication throughput
- GitHub Actions CI for `fmt`, `clippy`, tests, and release builds

## What it handles well

- Local symbolication from executables, object files, and dSYM payloads
- Inlined call-stack expansion for DWARF frames, on by default (like `atos -i`)
- Multi-address lookups in a single invocation
- Addresses from the command line, a file (`--input`), or stdin (streamed in `text` and `json-lines` modes)
- `.dSYM` bundle directories, or a directory searched by `--uuid` / build-id
- Separate ELF debug files via CRC-checked `.gnu_debuglink`, build-id, or the debuginfod cache
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

- `-f, --file-offsets`: treat each address as an offset from the image's `__TEXT` base, exactly like Apple `atos -offset`. The offset is rebased onto the `__TEXT` vmaddr; `--load-address` is **ignored** in this mode (a static file offset carries no runtime slide). See [Address modes](#address-modes).
- `-a, --arch <ARCH>`: choose a Mach-O slice in a fat binary
- `--uuid <UUID>`: choose a Mach-O slice by UUID, or select a file from a directory by UUID/build-id
- `-i, --input <FILE>`: read addresses from a file (defaults to stdin when no addresses are given)
- `--debug-dir <DIR>`: extra root to search for separate ELF debug files (repeatable)
- `--format <text|json|json-pretty|json-lines>`: select output format (`json-lines` emits one ndjson object per address and streams in input mode)
- `-v, --verbose`: print resolver diagnostics to stderr

### Address modes

`atosl` accepts addresses in two interpretations, matching the two `atos` workflows:

| Mode | Flag | Input meaning | Lookup address | `atos` equivalent |
| --- | --- | --- | --- | --- |
| Load-address (default) | _none_ | A runtime/virtual address as seen in a crash report | `address − load_address + __TEXT vmaddr` | `atos -l <load>` |
| File offset | `-f` / `--file-offsets` | An offset from the start of the image (`__TEXT` base) | `address + __TEXT vmaddr` | `atos -offset <off>` |

In the default mode you pass the load address that the image was mapped at (from the crash report's binary images section) and the runtime addresses. In file-offset mode the address is a static offset, so `--load-address` does not apply and is ignored. Inline frames are expanded in both modes (see below).

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

Stream one JSON object per address (ndjson), e.g. piping a crash log's addresses:

```bash
cat addresses.txt | atosl -o MyApp.app.dSYM -l 0x100000000 --format json-lines
```

Symbolize a file offset (equivalent to `atos -offset 0x4660`):

```bash
atosl -o MyApp.app.dSYM -l 0 -f 0x4660
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

## Inline frames

When DWARF describes inlined functions, `atosl` expands the full inline call
stack by default, innermost frame first — the same result Apple `atos` produces
with its `-i` / `--inlineFrames` flag (atos prints only the outermost frame
without it). For example, an address inside a function that inlined two helpers
prints:

```text
leaf_inline (in MyApp) (helpers.c:5)
mid_inline (in MyApp) (helpers.c:10)
outer (in MyApp) (outer.c:15)
```

In JSON output the inner frames appear under `inlined_by` on the resolved frame.

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
    input: None,
    debug_dirs: Vec::new(),
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

In addition, `tests/atos_differential.rs` builds a real Mach-O + dSYM on the
host and asserts that `atosl` agrees with Apple's `/usr/bin/atos` frame-for-frame
for DWARF resolution, inline-frame expansion, and `-f` (`atos -offset`) mode.
These tests are skipped automatically when not running on macOS or when `atos`
is unavailable, so they are a no-op on Linux CI.

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

Release steps are documented in [RELEASING.md](RELEASING.md).
For a one-command release flow, run `./deploy.sh [patch|minor|major|X.Y.Z]`.

## Known limitations

- This is still not a 1:1 clone of Apple's `atos`
- Symbolication quality depends on the symbol and DWARF data in the target binary
- Mach-O workflows remain the primary design target; other object formats work best when symbols are present
- Apple UUIDs and dSYM layouts are covered in tests, but real crash-log ingestion is still out of scope
- The Mach-O **debug map is not followed**. When you build with `-g` but do not run `dsymutil`, the executable keeps only `N_OSO` stabs pointing at the original `.o` files, and the line-table DWARF lives in those objects. Apple `atos` walks that debug map to recover source lines; `atosl` does not, so it falls back to the symbol table (`symbol + offset`) for such binaries. Point `atosl` at a generated `.dSYM` (or an object that embeds DWARF) to get source locations.
- Source paths are printed in full. Apple `atos` prints only the file name unless given `-fullPath`; `atosl` always prints the path as recorded in the DWARF line table.

## License

MIT. See `LICENSE`.
