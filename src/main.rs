//
// written by everettjf
// email : everettjf@live.com
// created at 2022-01-01
//
mod atosl;

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

    /// Enable debug mode with extra output
    #[clap(short, default_value_t = 0)]
    debug: u8,
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
    let debug_mode = args.debug != 0;

    let result = atosl::print_addresses(
        &object_path,
        args.load_address,
        args.addresses,
        debug_mode,
    );
    match result {
        Ok(..) => {}
        Err(err) => println!("{}", err),
    }
}
