//! `atosl` exposes a small Rust API around the CLI's symbolication engine.
//!
//! It is designed for local symbolication workflows where a caller already has
//! a binary or dSYM path and wants structured per-address results instead of
//! parsing terminal output.
//!
//! ```no_run
//! use atosl::{atosl, OutputFormat, SymbolizeOptions};
//!
//! let report = atosl::symbolize_path(&SymbolizeOptions {
//!     object_path: "MyApp.app/MyApp".into(),
//!     load_address: 0x1000_0000,
//!     addresses: vec![0x1000_1234],
//!     verbose: false,
//!     file_offsets: false,
//!     arch: Some("arm64".to_string()),
//!     uuid: None,
//!     format: OutputFormat::Json,
//! })?;
//!
//! println!("{:#?}", report.frames);
//! # Ok::<(), anyhow::Error>(())
//! ```

#![deny(unsafe_op_in_unsafe_fn)]

pub mod atosl;
pub mod demangle;

pub use atosl::{
    OutputFormat, ResolverKind, SelectedSlice, SourceLocation, SymbolizeOptions, SymbolizeOutcome,
    SymbolizeReport, SymbolizedFrame,
};
