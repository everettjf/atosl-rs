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
    if address.starts_with("0x") {
        let value = address.trim_start_matches("0x");
        let value = u64::from_str_radix(value, 16)?;
        Ok(value)
    } else {
        let value = address.parse::<u64>()?;
        Ok(value)
    }
}

fn main() {
    let args = Args::parse();
    let object_path = args.object_path.into_os_string().into_string().unwrap();

    let result = atosl::print_addresses(
        &object_path,
        args.load_address,
        args.addresses,
        args.verbose,
        args.file_offset_type,
    );
    match result {
        Ok(..) => {}
        Err(err) => println!("{}", err),
    }
}
