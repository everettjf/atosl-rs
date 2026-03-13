use assert_cmd::Command;
use object::{Object, ObjectSection, ObjectSegment, ObjectSymbol};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

#[test]
fn apple_macho_goldens_match() -> anyhow::Result<()> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }

    let fixture = AppleFixture::build()?;

    let alpha = fixture.symbol_address("golden_alpha")?;
    let beta = fixture.symbol_address("golden_beta")?;
    let load_address = fixture.load_address()?;

    let dwarf_output = symbolize(fixture.dsym_payload_path(), load_address, &[alpha, beta])?;
    let symbols_output = symbolize(fixture.stripped_binary_path(), load_address, &[alpha, beta])?;
    let dwarf_json = symbolize_json(
        fixture.dsym_payload_path(),
        load_address,
        &[alpha, beta],
        &[],
    )?;
    let dwarf_verbose = symbolize_verbose(
        fixture.dsym_payload_path(),
        load_address,
        &[alpha, beta],
        &[],
    )?;
    let symbols_json = symbolize_json(
        fixture.stripped_binary_path(),
        load_address,
        &[alpha, beta],
        &[],
    )?;

    assert_or_update(
        repo_root().join("tests/golden/apple/macho_dwarf.txt"),
        &dwarf_output,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/macho_symbols.txt"),
        &symbols_output,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/macho_dwarf.json"),
        &dwarf_json,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/macho_dwarf.verbose"),
        &dwarf_verbose,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/macho_symbols.json"),
        &symbols_json,
    )?;

    Ok(())
}

#[test]
fn apple_fat_macho_goldens_match() -> anyhow::Result<()> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }

    let fixture = AppleFatFixture::build()?;

    let arm64 = fixture.slice("arm64")?;
    let x86_64 = fixture.slice("x86_64")?;

    let arm64_output = symbolize_with_args(
        fixture.dsym_payload_path(),
        arm64.load_address,
        &[arm64.alpha, arm64.beta],
        &["--arch", "arm64"],
    )?;
    let x86_64_output = symbolize_with_args(
        fixture.dsym_payload_path(),
        x86_64.load_address,
        &[x86_64.alpha, x86_64.beta],
        &["--arch", "x86_64"],
    )?;
    let uuid_output = symbolize_with_args(
        fixture.dsym_payload_path(),
        x86_64.load_address,
        &[x86_64.alpha, x86_64.beta],
        &["--uuid", &x86_64.uuid],
    )?;
    let arm64_json = symbolize_json(
        fixture.dsym_payload_path(),
        arm64.load_address,
        &[arm64.alpha, arm64.beta],
        &["--arch", "arm64"],
    )?;
    let x86_64_json = symbolize_json(
        fixture.dsym_payload_path(),
        x86_64.load_address,
        &[x86_64.alpha, x86_64.beta],
        &["--arch", "x86_64"],
    )?;
    let uuid_json = symbolize_json(
        fixture.dsym_payload_path(),
        x86_64.load_address,
        &[x86_64.alpha, x86_64.beta],
        &["--uuid", &x86_64.uuid],
    )?;
    let arm64_verbose = symbolize_verbose(
        fixture.dsym_payload_path(),
        arm64.load_address,
        &[arm64.alpha, arm64.beta],
        &["--arch", "arm64"],
    )?;

    let ambiguous = Command::cargo_bin("atosl")?
        .arg("-o")
        .arg(fixture.dsym_payload_path())
        .arg("-l")
        .arg(format!("0x{:x}", arm64.load_address))
        .arg(format!("0x{:x}", arm64.alpha))
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let ambiguous = sanitize_uuids(&String::from_utf8(ambiguous)?);

    assert_or_update(
        repo_root().join("tests/golden/apple/fat_macho_arm64_dwarf.txt"),
        &arm64_output,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/fat_macho_x86_64_dwarf.txt"),
        &x86_64_output,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/fat_macho_uuid_dwarf.txt"),
        &uuid_output,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/fat_macho_ambiguous.stderr"),
        &ambiguous,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/fat_macho_arm64_dwarf.json"),
        &arm64_json,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/fat_macho_arm64_dwarf.verbose"),
        &arm64_verbose,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/fat_macho_x86_64_dwarf.json"),
        &x86_64_json,
    )?;
    assert_or_update(
        repo_root().join("tests/golden/apple/fat_macho_uuid_dwarf.json"),
        &uuid_json,
    )?;

    Ok(())
}

fn symbolize(object_path: &Path, load_address: u64, addresses: &[u64]) -> anyhow::Result<String> {
    symbolize_with_args(object_path, load_address, addresses, &[])
}

fn symbolize_with_args(
    object_path: &Path,
    load_address: u64,
    addresses: &[u64],
    extra_args: &[&str],
) -> anyhow::Result<String> {
    let mut cmd = Command::cargo_bin("atosl")?;
    cmd.arg("-o")
        .arg(object_path)
        .arg("-l")
        .arg(format!("0x{load_address:x}"));
    cmd.args(extra_args);

    for address in addresses {
        cmd.arg(format!("0x{address:x}"));
    }

    let output = cmd.assert().success().get_output().stdout.clone();
    Ok(String::from_utf8(output)?)
}

fn symbolize_json(
    object_path: &Path,
    load_address: u64,
    addresses: &[u64],
    extra_args: &[&str],
) -> anyhow::Result<String> {
    let mut cmd = Command::cargo_bin("atosl")?;
    cmd.arg("-o")
        .arg(object_path)
        .arg("-l")
        .arg(format!("0x{load_address:x}"))
        .arg("--format")
        .arg("json-pretty");
    cmd.args(extra_args);

    for address in addresses {
        cmd.arg(format!("0x{address:x}"));
    }

    let output = cmd.assert().success().get_output().stdout.clone();
    let mut json: Value = serde_json::from_slice(&output)?;
    sanitize_json(&mut json);
    Ok(format!("{}\n", serde_json::to_string_pretty(&json)?))
}

fn symbolize_verbose(
    object_path: &Path,
    load_address: u64,
    addresses: &[u64],
    extra_args: &[&str],
) -> anyhow::Result<String> {
    let mut cmd = Command::cargo_bin("atosl")?;
    cmd.arg("-o")
        .arg(object_path)
        .arg("-l")
        .arg(format!("0x{load_address:x}"))
        .arg("-v");
    cmd.args(extra_args);

    for address in addresses {
        cmd.arg(format!("0x{address:x}"));
    }

    let output = cmd.assert().success().get_output().stderr.clone();
    let output = String::from_utf8(output)?;
    Ok(sanitize_verbose(&output))
}

fn assert_or_update(path: PathBuf, actual: &str) -> anyhow::Result<()> {
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        fs::write(path, actual)?;
        return Ok(());
    }

    let expected = fs::read_to_string(&path)?;
    assert_eq!(expected, actual, "golden mismatch at {}", path.display());
    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

struct AppleFixture {
    _tempdir: TempDir,
    binary_path: PathBuf,
    stripped_binary_path: PathBuf,
    dsym_payload_path: PathBuf,
}

struct AppleFatFixture {
    _tempdir: TempDir,
    dsym_payload_path: PathBuf,
    slices: Vec<AppleSlice>,
}

#[derive(Clone)]
struct AppleSlice {
    arch: String,
    load_address: u64,
    alpha: u64,
    beta: u64,
    uuid: String,
}

impl AppleFixture {
    fn build() -> anyhow::Result<Self> {
        let tempdir = tempfile::tempdir()?;
        let repo_root = repo_root();
        let source_path = repo_root.join("tests/fixtures/apple/macho_golden.c");
        let object_path = tempdir.path().join("macho_golden.o");
        let binary_path = tempdir.path().join("macho_golden");
        let stripped_binary_path = tempdir.path().join("macho_golden.stripped");
        let dsym_path = tempdir.path().join("macho_golden.dSYM");
        let dsym_payload_path = dsym_path.join("Contents/Resources/DWARF/macho_golden");

        run_xcrun(&[
            "clang",
            "-g",
            "-O0",
            "-c",
            &format!("-fdebug-prefix-map={}={}", repo_root.display(), "."),
            source_path.to_str().unwrap(),
            "-o",
            object_path.to_str().unwrap(),
        ])?;

        run_xcrun(&[
            "clang",
            "-g",
            object_path.to_str().unwrap(),
            "-o",
            binary_path.to_str().unwrap(),
        ])?;

        run_xcrun(&[
            "dsymutil",
            binary_path.to_str().unwrap(),
            "-o",
            dsym_path.to_str().unwrap(),
        ])?;

        fs::copy(&binary_path, &stripped_binary_path)?;
        run_xcrun(&["strip", "-S", "-x", stripped_binary_path.to_str().unwrap()])?;

        Ok(Self {
            _tempdir: tempdir,
            binary_path,
            stripped_binary_path,
            dsym_payload_path,
        })
    }

    fn dsym_payload_path(&self) -> &Path {
        &self.dsym_payload_path
    }

    fn stripped_binary_path(&self) -> &Path {
        &self.stripped_binary_path
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
                    .map(|name| name.ends_with(symbol_name))
                    .unwrap_or(false)
            })
            .map(|symbol| symbol.address())
            .ok_or_else(|| anyhow::anyhow!("symbol not found: {symbol_name}"))
    }

    fn load_address(&self) -> anyhow::Result<u64> {
        let bytes = fs::read(&self.binary_path)?;
        let object = object::File::parse(bytes.as_slice())?;

        for segment in object.segments() {
            if let Some(name) = segment.name()? {
                if name == "__TEXT" {
                    return Ok(segment.address());
                }
            }
        }

        object
            .section_by_name("__text")
            .or_else(|| object.section_by_name(".text"))
            .map(|section| section.address())
            .ok_or_else(|| anyhow::anyhow!("text section not found"))
    }
}

fn run_xcrun(args: &[&str]) -> anyhow::Result<()> {
    let status = ProcessCommand::new("xcrun").args(args).status()?;
    anyhow::ensure!(status.success(), "xcrun command failed: {:?}", args);
    Ok(())
}

impl AppleFatFixture {
    fn build() -> anyhow::Result<Self> {
        let tempdir = tempfile::tempdir()?;
        let repo_root = repo_root();
        let source_path = repo_root.join("tests/fixtures/apple/macho_golden.c");
        let arm64_binary = tempdir.path().join("macho_golden.arm64");
        let x86_64_binary = tempdir.path().join("macho_golden.x86_64");
        let universal_binary = tempdir.path().join("macho_golden_fat");
        let dsym_path = tempdir.path().join("macho_golden_fat.dSYM");
        let dsym_payload_path = dsym_path.join("Contents/Resources/DWARF/macho_golden_fat");

        build_arch_binary(&source_path, &repo_root, "arm64", &arm64_binary)?;
        build_arch_binary(&source_path, &repo_root, "x86_64", &x86_64_binary)?;

        run_xcrun(&[
            "lipo",
            "-create",
            arm64_binary.to_str().unwrap(),
            x86_64_binary.to_str().unwrap(),
            "-output",
            universal_binary.to_str().unwrap(),
        ])?;

        run_xcrun(&[
            "dsymutil",
            universal_binary.to_str().unwrap(),
            "-o",
            dsym_path.to_str().unwrap(),
        ])?;

        Ok(Self {
            _tempdir: tempdir,
            dsym_payload_path,
            slices: vec![
                build_slice("arm64", arm64_binary)?,
                build_slice("x86_64", x86_64_binary)?,
            ],
        })
    }

    fn dsym_payload_path(&self) -> &Path {
        &self.dsym_payload_path
    }

    fn slice(&self, arch: &str) -> anyhow::Result<&AppleSlice> {
        self.slices
            .iter()
            .find(|slice| slice.arch == arch)
            .ok_or_else(|| anyhow::anyhow!("slice not found: {arch}"))
    }
}

fn build_arch_binary(
    source_path: &Path,
    repo_root: &Path,
    arch: &str,
    output: &Path,
) -> anyhow::Result<()> {
    let object_path = output.with_extension(format!("{arch}.o"));

    run_xcrun(&[
        "clang",
        "-arch",
        arch,
        "-g",
        "-O0",
        "-c",
        &format!("-fdebug-prefix-map={}={}", repo_root.display(), "."),
        source_path.to_str().unwrap(),
        "-o",
        object_path.to_str().unwrap(),
    ])?;

    run_xcrun(&[
        "clang",
        "-arch",
        arch,
        "-g",
        object_path.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ])
}

fn build_slice(arch: &str, binary_path: PathBuf) -> anyhow::Result<AppleSlice> {
    let bytes = fs::read(&binary_path)?;
    let object = object::File::parse(bytes.as_slice())?;
    let load_address = find_text_load_address(&object)?;
    let alpha = find_symbol_address(&object, "golden_alpha")?;
    let beta = find_symbol_address(&object, "golden_beta")?;
    let uuid = object
        .mach_uuid()?
        .map(format_uuid)
        .ok_or_else(|| anyhow::anyhow!("uuid not found for arch {arch}"))?;

    Ok(AppleSlice {
        arch: arch.to_string(),
        load_address,
        alpha,
        beta,
        uuid,
    })
}

fn find_symbol_address<'data>(
    object: &object::File<'data, &'data [u8]>,
    symbol_name: &str,
) -> anyhow::Result<u64> {
    object
        .symbols()
        .chain(object.dynamic_symbols())
        .find(|symbol| {
            symbol
                .name()
                .map(|name| name.ends_with(symbol_name))
                .unwrap_or(false)
        })
        .map(|symbol| symbol.address())
        .ok_or_else(|| anyhow::anyhow!("symbol not found: {symbol_name}"))
}

fn find_text_load_address<'data>(object: &object::File<'data, &'data [u8]>) -> anyhow::Result<u64> {
    for segment in object.segments() {
        if let Some(name) = segment.name()? {
            if name == "__TEXT" {
                return Ok(segment.address());
            }
        }
    }

    object
        .section_by_name("__text")
        .or_else(|| object.section_by_name(".text"))
        .map(|section| section.address())
        .ok_or_else(|| anyhow::anyhow!("text section not found"))
}

fn format_uuid(uuid: [u8; 16]) -> String {
    let hex = uuid
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

fn sanitize_uuids(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            if let Some((prefix, _)) = line.split_once("uuid=") {
                format!("{prefix}uuid=<UUID>")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + if input.ends_with('\n') { "\n" } else { "" }
}

fn sanitize_json(json: &mut Value) {
    if let Some(object_path) = json.get_mut("object_path") {
        *object_path = Value::String("<OBJECT_PATH>".to_string());
    }

    if let Some(selected_slice) = json
        .get_mut("selected_slice")
        .and_then(Value::as_object_mut)
    {
        if let Some(uuid) = selected_slice.get_mut("uuid") {
            *uuid = Value::String("<UUID>".to_string());
        }
    }
}

fn sanitize_verbose(output: &str) -> String {
    let mut normalized =
        sanitize_uuids(output).replace(&repo_root().display().to_string(), "<REPO_ROOT>");
    normalized = normalized
        .lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix("object: ") {
                if rest.contains('/') {
                    "object: <OBJECT_PATH>".to_string()
                } else {
                    line.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    normalized + if output.ends_with('\n') { "\n" } else { "" }
}
