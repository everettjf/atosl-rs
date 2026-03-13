use atosl::{atosl, OutputFormat, SymbolizeOptions};
use criterion::{criterion_group, criterion_main, Criterion};
use object::{Object, ObjectSection, ObjectSymbol};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn criterion_benchmark(c: &mut Criterion) {
    let fixture = BenchmarkFixture::build().expect("build fixture");
    let options = SymbolizeOptions {
        object_path: fixture.binary_path().to_path_buf(),
        load_address: fixture.load_address(),
        addresses: fixture.addresses().to_vec(),
        verbose: false,
        file_offsets: false,
        arch: None,
        uuid: None,
        format: OutputFormat::Json,
    };

    c.bench_function("symbolize_64_addresses", |b| {
        b.iter(|| {
            let report = atosl::symbolize_path(&options).expect("symbolize");
            assert_eq!(report.frames.len(), options.addresses.len());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

struct BenchmarkFixture {
    _tempdir: TempDir,
    binary_path: PathBuf,
    load_address: u64,
    addresses: Vec<u64>,
}

impl BenchmarkFixture {
    fn build() -> anyhow::Result<Self> {
        let tempdir = tempfile::tempdir()?;
        let source_path = tempdir.path().join("bench_fixture.c");
        let binary_path = tempdir.path().join("bench_fixture");
        fs::write(
            &source_path,
            r#"
int target_0(void) { return 0; }
int target_1(void) { return 1; }
int target_2(void) { return 2; }
int target_3(void) { return 3; }
int target_4(void) { return 4; }
int target_5(void) { return 5; }
int target_6(void) { return 6; }
int target_7(void) { return 7; }
int main(void) { return target_0() + target_7(); }
"#,
        )?;

        let status = Command::new("cc")
            .args([
                "-g",
                "-O0",
                source_path.to_str().expect("source path"),
                "-o",
                binary_path.to_str().expect("binary path"),
            ])
            .status()?;
        anyhow::ensure!(status.success(), "failed to compile benchmark fixture");

        let base_addresses = collect_target_addresses(&binary_path)?;
        let load_address = find_load_address(&binary_path)?;
        let addresses = base_addresses.iter().copied().cycle().take(64).collect();

        Ok(Self {
            _tempdir: tempdir,
            binary_path,
            load_address,
            addresses,
        })
    }

    fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    fn addresses(&self) -> &[u64] {
        &self.addresses
    }

    fn load_address(&self) -> u64 {
        self.load_address
    }
}

fn collect_target_addresses(binary_path: &Path) -> anyhow::Result<Vec<u64>> {
    let bytes = fs::read(binary_path)?;
    let object = object::File::parse(bytes.as_slice())?;

    let mut addresses = object
        .symbols()
        .filter_map(|symbol| {
            let name = symbol.name().ok()?;
            if name.starts_with("target_") {
                Some(symbol.address())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    addresses.sort_unstable();
    anyhow::ensure!(!addresses.is_empty(), "no benchmark symbols found");
    Ok(addresses)
}

fn find_load_address(binary_path: &Path) -> anyhow::Result<u64> {
    let bytes = fs::read(binary_path)?;
    let object = object::File::parse(bytes.as_slice())?;

    object
        .section_by_name("__text")
        .or_else(|| object.section_by_name(".text"))
        .map(|section| section.address())
        .ok_or_else(|| anyhow::anyhow!("text section not found"))
}
