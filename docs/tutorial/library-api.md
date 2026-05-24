---
title: Library API
layout: default
parent: Tutorials
nav_order: 8
---

# Library API (Rust)

`atosl` is also a crate. If you are building a crash-processing tool in Rust, you
can call the symbolication engine directly and get structured results instead of
parsing terminal output.

## Add the dependency

```toml
[dependencies]
atosl = "0.2"
```

## Symbolize

`SymbolizeOptions` implements `Default`, so set only the fields you care about
and let the rest fall back via `..Default::default()`:

```rust
use atosl::{OutputFormat, SymbolizeOptions};

let report = atosl::symbolize_path(&SymbolizeOptions {
    object_path: "MyApp.app.dSYM".into(),
    load_address: 0x1_0000_0000,
    addresses: vec![0x1_0000_1234],
    arch: Some("arm64".to_string()),
    format: OutputFormat::Json,
    ..Default::default()
})?;

for outcome in &report.frames {
    println!("{outcome:?}");
}
# Ok::<(), anyhow::Error>(())
```

Using `..Default::default()` also keeps your code compiling if a future release
adds new optional fields to `SymbolizeOptions`.

## The options

| Field | Type | Purpose |
| --- | --- | --- |
| `object_path` | `PathBuf` | Object, dSYM payload, `.dSYM` bundle, or directory |
| `load_address` | `u64` | Image load address (see [Address modes](address-modes)) |
| `addresses` | `Vec<u64>` | Addresses to resolve |
| `file_offsets` | `bool` | Legacy `-f` mode (`address − load_address`) |
| `inline_frames` | `bool` | Expand inline frames in text rendering |
| `arch` | `Option<String>` | Fat slice by architecture |
| `uuid` | `Option<String>` | Fat slice / directory file by UUID |
| `format` | `OutputFormat` | Output format used by the CLI emitters |
| `input` | `Option<PathBuf>` | Read addresses from a file |
| `debug_dirs` | `Vec<PathBuf>` | Extra roots for separate ELF debug files |
| `verbose` | `bool` | Resolver diagnostics |

## The result

`symbolize_path` returns a `SymbolizeReport`:

```rust
pub struct SymbolizeReport {
    pub object_path: String,
    pub object_name: String,
    pub selected_slice: Option<SelectedSlice>,
    pub frames: Vec<SymbolizeOutcome>,
}
```

Each `SymbolizeOutcome` is either `Resolved(SymbolizedFrame)` or `Unresolved
{ requested_address, error }`. A `SymbolizedFrame` carries the symbol, the
resolver that produced it (`dwarf` or `symbol_table`), the optional source
`location`, and the `inlined_by` chain. These map directly to the
[JSON field reference](output-formats#field-reference).

> The report always contains the full inline chain in `inlined_by`, regardless
> of the `inline_frames` flag — that flag only affects the CLI's text rendering.

## Stability

`SymbolizeOptions` deriving `Default` means new optional fields can be added
without breaking callers that use `..Default::default()`. See the project's
[release notes on API compatibility](https://github.com/everettjf/atosl-rs/blob/main/RELEASING.md#public-api-compatibility).
