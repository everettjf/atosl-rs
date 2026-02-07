//
// written by everettjf
// email : everettjf@live.com
// created at 2022-01-01
//
mod atosl;
mod demangle;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Symbol file path or binary file path
    #[clap(short, parse(from_os_str))]
    object_path: PathBuf,

    /// Load address of binary image
    #[clap(short, parse(try_from_str = parse_address_string))]
    load_address: u64,

    /// Addresses need to translate
    #[clap(parse(try_from_str = parse_address_string))]
    addresses: Vec<u64>,

    /// Enable verbose mode with extra output
    #[clap(short)]
    verbose: bool,

    /// Addresses are file offsets (ignore vmaddr in __TEXT or other executable segment)
    #[clap(short)]
    file_offset_type: bool,
}

fn parse_address_string(address: &str) -> Result<u64, anyhow::Error> {
    if let Some(value) = address
        .strip_prefix("0x")
        .or_else(|| address.strip_prefix("0X"))
    {
        let value = u64::from_str_radix(value, 16)?;
        Ok(value)
    } else {
        let value = address.parse::<u64>()?;
        Ok(value)
    }
}

fn main() {
    let args = Args::parse();
    let result = atosl::print_addresses(
        &args.object_path,
        args.load_address,
        &args.addresses,
        args.verbose,
        args.file_offset_type,
    );
    match result {
        Ok(..) => {}
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
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
