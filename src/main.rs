use atosl::{OutputFormat, SymbolizeOptions};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process;

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

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
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

    /// Treat addresses as file offsets: the lookup uses `address -
    /// load-address` directly, without re-basing onto the __TEXT vmaddr. To
    /// reproduce `atos -offset N`, use the default mode instead: `atosl -l 0 N`.
    #[arg(short = 'f', long = "file-offsets")]
    file_offset_type: bool,

    /// Expand inlined functions into the full call stack (innermost first),
    /// like `atos -i`. Off by default, which prints only the outermost frame.
    /// JSON output always includes inline frames under `inlined_by`.
    #[arg(long = "inline-frames")]
    inline_frames: bool,

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
    let args = Args::parse();
    let options = SymbolizeOptions {
        object_path: args.object_path,
        load_address: args.load_address,
        addresses: args.addresses,
        verbose: args.verbose,
        file_offsets: args.file_offset_type,
        inline_frames: args.inline_frames,
        arch: args.arch,
        uuid: args.uuid,
        format: args.format.into(),
        input: args.input,
        debug_dirs: args.debug_dir,
    };

    let exit_code = match atosl::atosl::run(options) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err:#}");
            1
        }
    };

    process::exit(exit_code);
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
