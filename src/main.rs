use atosl::{CrashSymbolizeOptions, OutputFormat, SymbolizeOptions};
use clap::{Parser, ValueEnum};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process;
use std::{fs, io};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum CliOutputFormat {
    Text,
    Json,
    JsonPretty,
    JsonLines,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(value: CliOutputFormat) -> Self {
        match value {
            CliOutputFormat::Text => OutputFormat::Text,
            CliOutputFormat::Json => OutputFormat::Json,
            CliOutputFormat::JsonPretty => OutputFormat::JsonPretty,
            CliOutputFormat::JsonLines => OutputFormat::JsonLines,
        }
    }
}

/// Symbolicate a whole crash report (.ips or legacy .crash text).
///
/// Invoked as `atosl crash <REPORT>`; the `crash` token is routed before the
/// default address-symbolication parser so it never collides with positional
/// addresses.
#[derive(Parser, Debug)]
#[command(name = "atosl crash", about)]
struct CrashArgs {
    /// Crash report path (.ips or .crash); reads stdin when omitted
    crash_path: Option<PathBuf>,

    /// Directory of dSYMs/binaries to match report images by UUID (repeatable)
    #[arg(short = 'd', long = "dsym-dir")]
    dsym_dir: Vec<PathBuf>,

    /// Write the symbolicated report here (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Enable verbose diagnostics
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(
    after_help = "Run `atosl crash <REPORT> --dsym-dir <DIR>` to symbolicate a whole crash report."
)]
struct SymbolizeArgs {
    /// Symbol file path or binary file path
    #[arg(short = 'o', long = "object", value_name = "OBJECT_PATH")]
    object_path: PathBuf,

    /// Load address of binary image
    #[arg(short = 'l', long = "load-address", value_parser = parse_address_string)]
    load_address: u64,

    /// Addresses that should be symbolized. When omitted, addresses are read
    /// from --input or stdin (one or more per line, whitespace-separated).
    #[arg(value_parser = parse_address_string)]
    addresses: Vec<u64>,

    /// Read addresses from this file instead of the command line (use stdin
    /// when neither addresses nor this flag are given)
    #[arg(short = 'i', long = "input")]
    input: Option<PathBuf>,

    /// Extra directory to search for separate ELF debug files (repeatable)
    #[arg(long = "debug-dir")]
    debug_dir: Vec<PathBuf>,

    /// Enable verbose diagnostics
    #[arg(short, long)]
    verbose: bool,

    /// Treat addresses as file offsets
    #[arg(short = 'f', long = "file-offsets")]
    file_offset_type: bool,

    /// Select architecture for Mach-O universal/fat files
    #[arg(short = 'a', long)]
    arch: Option<String>,

    /// Select Mach-O slice by UUID
    #[arg(long)]
    uuid: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Text)]
    format: CliOutputFormat,
}

fn parse_address_string(address: &str) -> Result<u64, String> {
    if let Some(value) = address
        .strip_prefix("0x")
        .or_else(|| address.strip_prefix("0X"))
    {
        u64::from_str_radix(value, 16).map_err(|err| err.to_string())
    } else {
        address.parse::<u64>().map_err(|err| err.to_string())
    }
}

fn main() {
    // Route `atosl crash ...` to the crash parser before the default parser, so
    // the report path never competes with positional addresses.
    let mut argv: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let is_crash = argv.get(1).map(|arg| arg == "crash").unwrap_or(false);

    let result = if is_crash {
        argv.remove(1);
        run_crash(CrashArgs::parse_from(argv))
    } else {
        run_symbolize(SymbolizeArgs::parse())
    };

    let exit_code = match result {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err:#}");
            1
        }
    };

    process::exit(exit_code);
}

fn run_symbolize(args: SymbolizeArgs) -> anyhow::Result<i32> {
    let options = SymbolizeOptions {
        object_path: args.object_path,
        load_address: args.load_address,
        addresses: args.addresses,
        verbose: args.verbose,
        file_offsets: args.file_offset_type,
        arch: args.arch,
        uuid: args.uuid,
        format: args.format.into(),
        input: args.input,
        debug_dirs: args.debug_dir,
    };

    atosl::atosl::run(options)
}

fn run_crash(args: CrashArgs) -> anyhow::Result<i32> {
    use anyhow::Context as _;

    let input = match &args.crash_path {
        Some(path) => fs::read_to_string(path)
            .with_context(|| format!("failed to read crash report: {}", path.display()))?,
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("failed to read crash report from stdin")?;
            buffer
        }
    };

    let report = atosl::symbolicate(
        &input,
        &CrashSymbolizeOptions {
            dsym_dirs: args.dsym_dir,
            verbose: args.verbose,
        },
    )?;

    match &args.output {
        Some(path) => fs::write(path, report)
            .with_context(|| format!("failed to write symbolicated report: {}", path.display()))?,
        None => io::stdout()
            .write_all(report.as_bytes())
            .context("failed to write symbolicated report")?,
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::parse_address_string;

    #[test]
    fn parse_hex_address() {
        assert_eq!(parse_address_string("0x10").unwrap(), 16);
        assert_eq!(parse_address_string("0Xff").unwrap(), 255);
    }

    #[test]
    fn parse_decimal_address() {
        assert_eq!(parse_address_string("42").unwrap(), 42);
    }

    #[test]
    fn parse_invalid_address() {
        assert!(parse_address_string("not_a_number").is_err());
    }
}
