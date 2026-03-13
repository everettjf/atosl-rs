use assert_cmd::Command;
use object::{Object, ObjectSection, ObjectSymbol};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

#[test]
fn cli_emits_json_for_resolved_symbol() {
    let fixture = Fixture::build().unwrap();
    let address = fixture.symbol_address("fixture_target").unwrap();

    let output = Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "-o",
            fixture.binary_path().to_str().unwrap(),
            "-l",
            &format!("0x{:x}", fixture.load_address().unwrap()),
            "--format",
            "json",
            &format!("0x{address:x}"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(parsed["frames"][0]["status"], "resolved");
    assert_eq!(parsed["object_name"], "fixture_bin");
    assert_eq!(parsed["frames"][0]["requested_address"], address);
}

#[test]
fn cli_emits_text_for_resolved_symbol() {
    let fixture = Fixture::build().unwrap();
    let address = fixture.symbol_address("fixture_target").unwrap();

    Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "-o",
            fixture.binary_path().to_str().unwrap(),
            "-l",
            &format!("0x{:x}", fixture.load_address().unwrap()),
            &format!("0x{address:x}"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("(in fixture_bin)"));
}

struct Fixture {
    _tempdir: TempDir,
    binary_path: PathBuf,
}

impl Fixture {
    fn build() -> anyhow::Result<Self> {
        let tempdir = tempfile::tempdir()?;
        let source_path = tempdir.path().join("fixture.c");
        let binary_path = tempdir.path().join("fixture_bin");
        fs::write(
            &source_path,
            r#"
int fixture_target(void) {
    return 42;
}

int main(void) {
    return fixture_target();
}
"#,
        )?;

        let status = ProcessCommand::new("cc")
            .args([
                "-g",
                "-O0",
                source_path.to_str().unwrap(),
                "-o",
                binary_path.to_str().unwrap(),
            ])
            .status()?;

        anyhow::ensure!(status.success(), "failed to build test fixture");

        Ok(Self {
            _tempdir: tempdir,
            binary_path,
        })
    }

    fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    fn symbol_address(&self, symbol_name: &str) -> anyhow::Result<u64> {
        let bytes = fs::read(&self.binary_path)?;
        let object = object::File::parse(bytes.as_slice())?;

        object
            .symbols()
            .chain(object.dynamic_symbols())
            .find(|symbol| {
                symbol
                    .name()
                    .map(|name| name.contains(symbol_name))
                    .unwrap_or(false)
            })
            .map(|symbol| symbol.address())
            .ok_or_else(|| anyhow::anyhow!("symbol not found: {symbol_name}"))
    }

    fn load_address(&self) -> anyhow::Result<u64> {
        let bytes = fs::read(&self.binary_path)?;
        let object = object::File::parse(bytes.as_slice())?;

        object
            .section_by_name("__text")
            .or_else(|| object.section_by_name(".text"))
            .map(|section| section.address())
            .ok_or_else(|| anyhow::anyhow!("text section not found"))
    }
}
