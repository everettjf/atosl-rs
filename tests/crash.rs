use assert_cmd::Command;
use object::{Object, ObjectSection, ObjectSymbol};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;

const BUILD_ID: &str = "abcdef0123456789abcdef0123456789abcdef01";
const BASE: u64 = 0x1_0000_0000;

#[test]
fn symbolicates_legacy_text_crash() {
    if !cfg!(target_os = "linux") {
        return;
    }

    let tempdir = tempfile::tempdir().unwrap();
    let dsym_dir = tempdir.path().join("symbols");
    fs::create_dir_all(&dsym_dir).unwrap();
    let bin = dsym_dir.join("app");
    build_fixture(tempdir.path(), &bin);

    let image_offset = symbol_addr(&bin, "fixture_target") - text_addr(&bin);
    let runtime = BASE + image_offset;

    let report = format!(
        "Thread 0 Crashed:\n\
         0   app   0x{runtime:016x}   0x{BASE:x} + {image_offset}\n\
         \n\
         Binary Images:\n\
         0x{BASE:x} - 0x{end:x}  app arm64  <{BUILD_ID}>  /tmp/app\n",
        end = BASE + 0x1000,
    );
    let report_path = tempdir.path().join("crash.crash");
    fs::write(&report_path, report).unwrap();

    Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "crash",
            report_path.to_str().unwrap(),
            "--dsym-dir",
            dsym_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("0   app   "))
        .stdout(predicates::str::contains("fixture_target"));
}

#[test]
fn symbolicates_ips_crash() {
    if !cfg!(target_os = "linux") {
        return;
    }

    let tempdir = tempfile::tempdir().unwrap();
    let dsym_dir = tempdir.path().join("symbols");
    fs::create_dir_all(&dsym_dir).unwrap();
    let bin = dsym_dir.join("app");
    build_fixture(tempdir.path(), &bin);

    let image_offset = symbol_addr(&bin, "fixture_target") - text_addr(&bin);

    let body = serde_json::json!({
        "usedImages": [
            { "base": BASE, "uuid": BUILD_ID, "name": "app", "arch": "arm64" }
        ],
        "threads": [
            { "frames": [ { "imageIndex": 0, "imageOffset": image_offset } ] }
        ]
    });
    let report = format!("{{\"app_name\":\"app\"}}\n{body}");
    let report_path = tempdir.path().join("crash.ips");
    fs::write(&report_path, report).unwrap();

    let output = Command::cargo_bin("atosl")
        .unwrap()
        .args([
            "crash",
            report_path.to_str().unwrap(),
            "--dsym-dir",
            dsym_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let body_line = text
        .lines()
        .nth(1)
        .expect("ips output should keep a body line");
    let parsed: Value = serde_json::from_str(body_line).unwrap();
    assert_eq!(
        parsed["threads"][0]["frames"][0]["symbol"],
        Value::String("fixture_target".to_string())
    );
}

fn build_fixture(workdir: &Path, out: &Path) {
    let src = workdir.join("f.c");
    fs::write(
        &src,
        "int fixture_target(void){return 7;}\nint main(void){return fixture_target();}\n",
    )
    .unwrap();
    let status = ProcessCommand::new("cc")
        .args([
            "-g",
            "-O0",
            &format!("-Wl,--build-id=0x{BUILD_ID}"),
            src.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "failed to build fixture {}",
        out.display()
    );
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
