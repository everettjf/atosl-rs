//! `atosl` exposes a small Rust API around the CLI's symbolication engine.
//!
//! It is designed for local symbolication workflows where a caller already has
//! a binary or dSYM path and wants structured per-address results instead of
//! parsing terminal output.
//!
//! ```no_run
//! use atosl::{atosl, OutputFormat, SymbolizeOptions};
//!
//! // Only set the fields you care about; the rest fall back to defaults.
//! let report = atosl::symbolize_path(&SymbolizeOptions {
//!     object_path: "MyApp.app/MyApp".into(),
//!     load_address: 0x1000_0000,
//!     addresses: vec![0x1000_1234],
//!     arch: Some("arm64".to_string()),
//!     format: OutputFormat::Json,
//!     ..Default::default()
//! })?;
//!
//! println!("{:#?}", report.frames);
//! # Ok::<(), anyhow::Error>(())
//! ```

#![deny(unsafe_op_in_unsafe_fn)]

pub mod atosl;
pub mod crash;
pub mod demangle;

pub use atosl::{
    InlineFrame, OutputFormat, ResolverKind, SelectedSlice, SourceLocation, SymbolizeOptions,
    SymbolizeOutcome, SymbolizeReport, SymbolizedFrame,
};
pub use crash::{symbolicate, CrashSymbolizeOptions};
