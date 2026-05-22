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

#[test]
fn cli_resolves_dsym_bundle_directory() {
    let fixture = Fixture::build().unwrap();
    let address = fixture.symbol_address("fixture_target").unwrap();

    let dwarf_dir = fixture
        .binary_path()
        .parent()
        .unwrap()
        .join("Fixture.dSYM/Contents/Resources/DWARF");
    fs::create_dir_all(&dwarf_dir).unwrap();
    let payload = dwarf_dir.join("fixture_bin");
    fs::copy(fixture.binary_path(), &payload).unwrap();
    let bundle = fixture.binary_path().parent().unwrap().join("Fixture.dSYM");

    Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "-o",
            bundle.to_str().unwrap(),
            "-l",
            &format!("0x{:x}", fixture.load_address().unwrap()),
            &format!("0x{address:x}"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("(in fixture_bin)"));
}

#[test]
fn cli_reads_addresses_from_stdin() {
    let fixture = Fixture::build().unwrap();
    let address = fixture.symbol_address("fixture_target").unwrap();

    Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "-o",
            fixture.binary_path().to_str().unwrap(),
            "-l",
            &format!("0x{:x}", fixture.load_address().unwrap()),
        ])
        .write_stdin(format!("0x{address:x}\n"))
        .assert()
        .success()
        .stdout(predicates::str::contains("(in fixture_bin)"));
}

#[test]
fn cli_reads_addresses_from_input_file() {
    let fixture = Fixture::build().unwrap();
    let address = fixture.symbol_address("fixture_target").unwrap();

    let input_path = fixture.binary_path().parent().unwrap().join("addrs.txt");
    fs::write(&input_path, format!("0x{address:x}\n")).unwrap();

    Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "-o",
            fixture.binary_path().to_str().unwrap(),
            "-l",
            &format!("0x{:x}", fixture.load_address().unwrap()),
            "--input",
            input_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("(in fixture_bin)"));
}

#[test]
fn cli_finds_object_in_directory_by_build_id() {
    // build-id is an ELF/GNU-ld concept; skip on non-Linux toolchains.
    if !cfg!(target_os = "linux") {
        return;
    }

    let tempdir = tempfile::tempdir().unwrap();
    let src = tempdir.path().join("f.c");
    fs::write(
        &src,
        "int fixture_target(void){return 7;}\nint main(void){return fixture_target();}\n",
    )
    .unwrap();

    let dir = tempdir.path().join("symbols");
    fs::create_dir_all(&dir).unwrap();
    let alpha = dir.join("alpha");
    let beta = dir.join("beta");
    let id_beta = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    build_with_build_id(&src, &alpha, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    build_with_build_id(&src, &beta, id_beta);

    let address = symbol_addr(&beta, "fixture_target");
    let load = text_addr(&beta);

    Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "-o",
            dir.to_str().unwrap(),
            "--uuid",
            id_beta,
            "-l",
            &format!("0x{load:x}"),
            &format!("0x{address:x}"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("(in beta)"));
}

#[test]
fn cli_follows_gnu_debuglink_to_separate_debug_file() {
    // .gnu_debuglink + objcopy are ELF/binutils features; skip elsewhere.
    if !cfg!(target_os = "linux") {
        return;
    }

    let tempdir = tempfile::tempdir().unwrap();
    let src = tempdir.path().join("f.c");
    fs::write(
        &src,
        "int fixture_target(void){return 7;}\nint main(void){return fixture_target();}\n",
    )
    .unwrap();

    let bin = tempdir.path().join("app");
    let status = ProcessCommand::new("cc")
        .args([
            "-g",
            "-O0",
            src.to_str().unwrap(),
            "-o",
            bin.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    run_objcopy(&["--only-keep-debug", "app", "app.debug"], tempdir.path());
    run_objcopy(&["--strip-debug", "app"], tempdir.path());
    run_objcopy(&["--add-gnu-debuglink=app.debug", "app"], tempdir.path());

    let address = symbol_addr(&bin, "fixture_target");
    let load = text_addr(&bin);

    Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "-o",
            bin.to_str().unwrap(),
            "-l",
            &format!("0x{load:x}"),
            &format!("0x{address:x}"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("(in app.debug)"));
}

fn run_objcopy(args: &[&str], cwd: &Path) {
    let status = ProcessCommand::new("objcopy")
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap();
    assert!(status.success(), "objcopy {args:?} failed");
}

fn build_with_build_id(src: &Path, out: &Path, build_id: &str) {
    let status = ProcessCommand::new("cc")
        .args([
            "-g",
            "-O0",
            &format!("-Wl,--build-id=0x{build_id}"),
            src.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "failed to build {}", out.display());
}

fn symbol_addr(path: &Path, name: &str) -> u64 {
    let bytes = fs::read(path).unwrap();
    let object = object::File::parse(bytes.as_slice()).unwrap();
    object
        .symbols()
        .chain(object.dynamic_symbols())
        .find(|symbol| symbol.name().map(|n| n.contains(name)).unwrap_or(false))
        .map(|symbol| symbol.address())
        .unwrap()
}

fn text_addr(path: &Path) -> u64 {
    let bytes = fs::read(path).unwrap();
    let object = object::File::parse(bytes.as_slice()).unwrap();
    object
        .section_by_name(".text")
        .or_else(|| object.section_by_name("__text"))
        .map(|section| section.address())
        .unwrap()
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
