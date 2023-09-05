//
// written by everettjf
// email : everettjf@live.com
// created at 2022-01-01
//
mod atosl;
mod demangle;

use anyhow::Result;
use clap::Parser;
use std::io;
use std::io::{BufRead, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Symbol file path or binary file path
    #[clap(
    short,
    parse(from_os_str),
    default_value = "/Users/doude/code/rust/atosl-rs/examples/case1/RxDemo.app.dSYM/Contents/Resources/DWARF/RxDemo"
    )]
    object_path: PathBuf,

    /// Load address of binary image
    #[clap(short, parse(try_from_str = parse_address_string), default_value = "0x102c18000")]
    load_address: u64,

    /// Addresses need to translate
    #[clap(parse(try_from_str = parse_address_string), default_value = "0x0000000102c1ed50")]
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

//  atosl -o /Users/doude/code/rust/atosl-rs/examples/case1/RxDemo.app.dSYM/Contents/Resources/DWARF/RxDemo  -l 0x102c18000 0x0000000102c1ed50
fn main() {
    let args = Args::parse();
    let object_path = args.object_path.into_os_string().into_string().unwrap();
    let x = crate::atosl::init_file_obj(&object_path).unwrap();
    // 创建标准输入流的读取器和标准输出流
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let x: String = String::new();
    x.split("\n");
    loop {
        // 打印提示信息
        write!(stdout, "user：").expect("Failed to write to stdout");
        stdout.flush().expect("Failed to flush stdout");
        // 读取用户输入的内容
        let mut input = String::new();
        stdin
            .lock()
            .read_line(&mut input)
            .expect("Failed to read line");
        // 去除输入内容两端的空格和换行符
        let input = input.trim();
        let split: Vec<&str> = input.split_whitespace().collect();

        // 根据用户输入选择相应的操作
        match input {
            "exit" => {
                println!("对话结束");
                break;
            }
            _ => {
                if split.len() != 2 {
                    println!("got_err command args 「{}」 is invalid, just two hex data", input)
                } else {
                    let load_address = parse_address_string(split[0]);
                    let address_arr = parse_address_string(split[1]);
                    if load_address.is_ok() && address_arr.is_ok() {
                        vec![parse_address_string(split[1]).unwrap()];
                        let result = atosl::print_addresses(
                            load_address.unwrap(),
                            vec![address_arr.unwrap()],
                            args.verbose,
                            args.file_offset_type,
                        );
                        match result {
                            Ok(..) => {}
                            Err(err) => println!("{}", err),
                        }
                    } else {
                        println!("got_err command args  「{}」 ,「{}」  is invalid, ", split[0], split[1])
                    }
                }
            }
        }
    }
}
