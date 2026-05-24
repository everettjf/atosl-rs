//! Differential tests that compare `atosl` against Apple's own `/usr/bin/atos`.
//!
//! Unlike the golden tests (which pin `atosl`'s own output to checked-in
//! snapshots), these tests build a real Mach-O + dSYM on the host and assert
//! that `atosl` agrees with `atos` frame-for-frame. They are inherently
//! codegen-robust: both tools are handed the identical addresses, so the test
//! only fails when the two tools genuinely disagree, not when the compiler
//! lays the function out differently.
//!
//! The whole suite is skipped unless it is running on macOS with `atos`
//! available, so it is a no-op on Linux CI.

#![cfg(target_os = "macos")]

use assert_cmd::Command;
use object::{Object, ObjectSegment, ObjectSymbol};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

const ATOS: &str = "/usr/bin/atos";

/// Inline-heavy address inside `outer` must expand the same way under both
/// tools. `atosl` expands inline frames by default, which mirrors `atos -i`.
#[test]
fn matches_atos_on_dwarf_and_inline_frames() -> anyhow::Result<()> {
    let Some(fixture) = InlineFixture::build()? else {
        eprintln!("skipping: atos not available");
        return Ok(());
    };

    let outer = fixture.symbol_address("_outer")?;
    let main = fixture.symbol_address("_main")?;
    let load = fixture.text_vmaddr;

    // Probe every instruction-sized offset across `outer` (it holds the
    // inlined leaf/mid frames) plus the entry of `main`.
    let mut addresses: Vec<u64> = (outer..main).step_by(4).collect();
    addresses.push(main);

    for address in addresses {
        let ours = run_atosl(&[
            "-o",
            fixture.dsym_payload.to_str().unwrap(),
            "-l",
            &hex(load),
            &hex(address),
        ]);
        // `atos` only emits inline frames when asked with -i; atosl does so by
        // default, so we compare against `atos -i`.
        let theirs = run_atos(&[
            "-i",
            "-o",
            fixture.dsym_payload.to_str().unwrap(),
            "-l",
            &hex(load),
            &hex(address),
        ])?;

        assert_eq!(
            normalize_frames(&ours),
            normalize_frames(&theirs),
            "atosl/atos disagree at {}\n  atosl: {ours:?}\n  atos : {theirs:?}",
            hex(address),
        );
    }

    Ok(())
}

/// `atosl -f` treats the address as an offset from the image's __TEXT base,
/// which is exactly Apple `atos -offset`.
#[test]
fn matches_atos_offset_mode() -> anyhow::Result<()> {
    let Some(fixture) = InlineFixture::build()? else {
        eprintln!("skipping: atos not available");
        return Ok(());
    };

    let outer = fixture.symbol_address("_outer")?;
    let offset = outer - fixture.text_vmaddr;

    // The load address is irrelevant in offset mode and must be ignored, so we
    // deliberately pass a non-zero one to prove it does not change the result.
    let ours = run_atosl(&[
        "-o",
        fixture.dsym_payload.to_str().unwrap(),
        "-l",
        &hex(fixture.text_vmaddr),
        "-f",
        &hex(offset),
    ]);
    let theirs = run_atos(&[
        "-i",
        "-o",
        fixture.dsym_payload.to_str().unwrap(),
        "-offset",
        &hex(offset),
    ])?;

    assert_eq!(
        normalize_frames(&ours),
        normalize_frames(&theirs),
        "atosl -f / atos -offset disagree at offset {}\n  atosl: {ours:?}\n  atos : {theirs:?}",
        hex(offset),
    );

    Ok(())
}

fn hex(value: u64) -> String {
    format!("0x{value:x}")
}

fn run_atosl(args: &[&str]) -> String {
    let output = Command::cargo_bin("atosl")
        .unwrap()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

fn run_atos(args: &[&str]) -> anyhow::Result<String> {
    let output = ProcessCommand::new(ATOS).args(args).output()?;
    anyhow::ensure!(output.status.success(), "atos failed: {:?}", args);
    Ok(String::from_utf8(output.stdout)?)
}

/// Reduce a tool's output to a comparable shape: one entry per frame as
/// `symbol (basename:line)`, `symbol + offset`, or the raw line.
///
/// This intentionally drops the `(in <image>)` qualifier and the source
/// directory. atosl prints absolute source paths while atos prints just the
/// file name (without `-fullPath`); that cosmetic difference is not what these
/// tests are guarding.
fn normalize_frames(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(normalize_frame)
        .collect()
}

fn normalize_frame(line: &str) -> String {
    let Some(in_idx) = line.find(" (in ") else {
        // Raw address or anything else without the standard shape.
        return line.to_string();
    };
    let symbol = &line[..in_idx];

    // Trailing "(file:line)" wins when present.
    if line.ends_with(')') {
        if let Some(open) = line.rfind('(') {
            let inner = &line[open + 1..line.len() - 1];
            if let Some(colon) = inner.rfind(':') {
                let file = &inner[..colon];
                let lineno = &inner[colon..]; // includes ':'
                let base = file.rsplit('/').next().unwrap_or(file);
                return format!("{symbol} ({base}{lineno})");
            }
        }
    }

    // Symbol-table form: "symbol (in image) + 16".
    if let Some(plus) = line.find(" + ") {
        return format!("{symbol} {}", &line[plus + 1..]);
    }

    symbol.to_string()
}

struct InlineFixture {
    _tempdir: TempDir,
    binary: PathBuf,
    dsym_payload: PathBuf,
    text_vmaddr: u64,
}

impl InlineFixture {
    /// Returns `Ok(None)` when `atos` is not installed so callers can skip.
    fn build() -> anyhow::Result<Option<Self>> {
        if !Path::new(ATOS).exists() {
            return Ok(None);
        }

        let tempdir = tempfile::tempdir()?;
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source = repo_root.join("tests/fixtures/apple/inline_golden.c");
        let object = tempdir.path().join("inline_golden.o");
        let binary = tempdir.path().join("inline_golden");
        let dsym = tempdir.path().join("inline_golden.dSYM");
        let dsym_payload = dsym.join("Contents/Resources/DWARF/inline_golden");

        // -O2 so the always_inline helpers actually produce inline DWARF DIEs.
        run_xcrun(&[
            "clang",
            "-arch",
            "arm64",
            "-g",
            "-O2",
            "-c",
            source.to_str().unwrap(),
            "-o",
            object.to_str().unwrap(),
        ])?;
        run_xcrun(&[
            "clang",
            "-arch",
            "arm64",
            "-g",
            object.to_str().unwrap(),
            "-o",
            binary.to_str().unwrap(),
        ])?;
        run_xcrun(&[
            "dsymutil",
            binary.to_str().unwrap(),
            "-o",
            dsym.to_str().unwrap(),
        ])?;

        let text_vmaddr = read_text_vmaddr(&binary)?;

        Ok(Some(Self {
            _tempdir: tempdir,
            binary,
            dsym_payload,
            text_vmaddr,
        }))
    }

    fn symbol_address(&self, symbol_name: &str) -> anyhow::Result<u64> {
        let bytes = fs::read(&self.binary)?;
        let object = object::File::parse(bytes.as_slice())?;
        object
            .symbols()
            .chain(object.dynamic_symbols())
            .find(|symbol| {
                symbol
                    .name()
                    .map(|name| name == symbol_name)
                    .unwrap_or(false)
            })
            .map(|symbol| symbol.address())
            .ok_or_else(|| anyhow::anyhow!("symbol not found: {symbol_name}"))
    }
}

fn read_text_vmaddr(binary: &Path) -> anyhow::Result<u64> {
    let bytes = fs::read(binary)?;
    let object = object::File::parse(bytes.as_slice())?;
    for segment in object.segments() {
        if segment.name()? == Some("__TEXT") {
            return Ok(segment.address());
        }
    }
    anyhow::bail!("__TEXT segment not found")
}

fn run_xcrun(args: &[&str]) -> anyhow::Result<()> {
    let status = ProcessCommand::new("xcrun").args(args).status()?;
    anyhow::ensure!(status.success(), "xcrun command failed: {:?}", args);
    Ok(())
}
