//
// written by everettjf
// email : everettjf@live.com
// created at 2022-01-01
//
use crate::demangle;
use anyhow::{anyhow, Result};
use gimli::{DW_TAG_subprogram, DebugInfoOffset, Dwarf, EndianSlice, RunTimeEndian};
use object::{File, Object, ObjectSection, ObjectSegment};
use std::path::Path;
use std::{borrow, fs};
use std::io::Cursor;
use std::sync::Arc;


pub fn init_file_obj(object_path: &str) -> Result<(), anyhow::Error> {
    MAPPED_OBJECT_FILE.lock().unwrap();
    Ok(())
}

pub struct MappedObjectFile {
    mmap: memmap::Mmap,
    object_file: object::File<'static>,
    file_name: String,
}

use std::sync::Mutex;
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref MAPPED_OBJECT_FILE: Mutex<MappedObjectFile> = Mutex::new({
       // let key = env::var("KEY").unwrap();
        let object_path = env::var("atosl_resource_file").unwrap();
        let file = fs::File::open(&object_path).unwrap();
        let mmap = unsafe { memmap::Mmap::map(&file).unwrap() };
        let mmap = unsafe { memmap::Mmap::map(&file).unwrap() };
        let bytes = unsafe { std::slice::from_raw_parts(mmap.as_ptr(), mmap.len()) };
        let object = object::File::parse(bytes).unwrap();
        // Create a new file handle and mmap it for the object::File
        let file_for_object = fs::File::open(&object_path).unwrap();
        let mmap_for_object = unsafe { memmap::Mmap::map(&file_for_object).unwrap() };
        // let object = object::File::parse(mmap_for_object).unwrap();
        let object_filename = Path::new(&object_path)
            .file_name()
            .expect("file name error")
            .to_str()
            .expect("file name error(to_str)");
        let mapped_object_file = MappedObjectFile {
            mmap,
            object_file: object,
            file_name: object_filename.to_string()
        };
        mapped_object_file
    });
}
pub fn print_addresses(
    // object: &File,
    // object_filename: &str,
    load_address: u64,
    addresses: Vec<u64>,
    verbose: bool,
    file_offset_type: bool,
) -> Result<(), anyhow::Error> {
    let mut mapped_object_file = MAPPED_OBJECT_FILE.lock().unwrap();
    let object_filename = mapped_object_file.file_name.as_str();
    let object = &mapped_object_file.object_file;
    if is_object_dwarf(object) {
        if verbose {
            println!("dwarf");
        }
        dwarf_symbolize_addresses(
            object,
            object_filename,
            load_address,
            addresses,
            verbose,
            file_offset_type,
        )
    } else {
        if verbose {
            println!("symbols");
        }
        symbol_symbolize_addresses(
            object,
            object_filename,
            load_address,
            addresses,
            verbose,
            file_offset_type,
        )
    }
}

fn is_object_dwarf(object: &object::File) -> bool {
    if let Some(_) = object.section_by_name("__debug_line") {
        true
    } else {
        false
    }
}

fn symbol_symbolize_addresses(
    object: &object::File,
    object_filename: &str,
    load_address: u64,
    addresses: Vec<u64>,
    verbose: bool,
    file_offset_type: bool,
) -> Result<(), anyhow::Error> {
    // find vmaddr for __TEXT segment
    let mut segments = object.segments();
    let mut text_vmaddr = 0;
    while let Some(segment) = segments.next() {
        if let Some(name) = segment.name()? {
            if name == "__TEXT" {
                text_vmaddr = segment.address();
                break;
            }
        }
    }

    for address in addresses {
        if verbose {
            println!("---------------------------------------------");
            println!("BEGIN ADDRESS {} | {:016x}", address, address);
        }

        let symbol_result = symbol_symbolize_address(
            &object,
            object_filename,
            load_address,
            address,
            text_vmaddr,
            verbose,
            file_offset_type,
        );
        if verbose {
            println!("RESULT:")
        }
        match symbol_result {
            Ok(symbol) => println!("{}", symbol),
            Err(err) => println!("N/A - {}", err),
        };

        if verbose {
            println!("END ADDRESS {} | {:016x}", address, address);
        }
    }
    Ok(())
}

fn symbol_symbolize_address(
    object: &object::File,
    object_filename: &str,
    load_address: u64,
    address: u64,
    text_vmaddr: u64,
    _verbose: bool,
    file_offset_type: bool,
) -> Result<String, anyhow::Error> {
    let search_address: u64 = if file_offset_type {
        address - load_address
    } else {
        address - load_address + text_vmaddr
    };

    let symbols = object.symbol_map();
    let found_symbol = symbols.get(search_address);

    if let Some(found_symbol) = found_symbol {
        // expect format
        // main (in BinaryName)
        let offset = search_address - found_symbol.address();
        let demangled_name = demangle::demangle_symbol(found_symbol.name());
        let symbolize_result = format!("{} (in {}) + {}", demangled_name, object_filename, offset);
        return Ok(symbolize_result);
    }

    Err(anyhow!("failed search symbol"))
}

fn dwarf_symbolize_addresses(
    object: &object::File,
    object_filename: &str,
    load_address: u64,
    addresses: Vec<u64>,
    verbose: bool,
    file_offset_type: bool,
) -> Result<(), anyhow::Error> {
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };
    let dwarf_cow = gimli::Dwarf::load(|section_id| -> Result<borrow::Cow<[u8]>, gimli::Error> {
        // println!("section id = {}", section_id.name());
        match object.section_by_name(section_id.name()) {
            Some(ref section) => Ok(section
                .uncompressed_data()
                .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
            None => Ok(borrow::Cow::Borrowed(&[][..])),
        }
    })?;
    let dwarf = dwarf_cow.borrow(|section| gimli::EndianSlice::new((&*section).as_ref(), endian));

    // find vmaddr for __TEXT segment
    let mut segments = object.segments();
    let mut text_vmaddr = 0;
    while let Some(segment) = segments.next() {
        if let Some(name) = segment.name()? {
            if name == "__TEXT" {
                text_vmaddr = segment.address();
                break;
            }
        }
    }

    for address in addresses {
        if verbose {
            println!("---------------------------------------------");
            println!("BEGIN ADDRESS {} | {:016x}", address, address);
        }

        let symbol_result = dwarf_symbolize_address(
            &dwarf,
            object_filename,
            load_address,
            address,
            text_vmaddr,
            verbose,
            file_offset_type,
        );
        if verbose {
            println!("RESULT:")
        }
        match symbol_result {
            Ok(symbol) => println!("{}", symbol),
            Err(_) => {
                // downgrade to symbol table search
                let symbol_result = symbol_symbolize_address(
                    &object,
                    object_filename,
                    load_address,
                    address,
                    text_vmaddr,
                    verbose,
                    file_offset_type,
                );
                match symbol_result {
                    Ok(symbol) => println!("{}", symbol),
                    Err(err) => println!("N/A - {}", err),
                };
            }
        };

        if verbose {
            println!("END ADDRESS {} | {:016x}", address, address);
        }
    }
    Ok(())
}

fn dwarf_symbolize_address(
    dwarf: &Dwarf<EndianSlice<RunTimeEndian>>,
    object_filename: &str,
    load_address: u64,
    address: u64,
    text_vmaddr: u64,
    verbose: bool,
    file_offset_type: bool,
) -> Result<String, anyhow::Error> {
    let search_address: u64 = if file_offset_type {
        address - load_address
    } else {
        address - load_address + text_vmaddr
    };

    // aranges
    let mut debug_info_offset: Option<DebugInfoOffset> = None;
    let mut arange_headers = dwarf.debug_aranges.headers();
    while let Some(header) = arange_headers.next()? {
        let mut found_address = false;
        let mut arange_entries = header.entries();
        while let Some(entry) = arange_entries.next()? {
            // println!("- | entry: address=0x{:016x} length=0x{:016x} segment=0x{:016x}", entry.address(), entry.length(), entry.segment().unwrap_or(0));
            let begin = entry.address();
            let end = entry.address() + entry.length();
            if search_address >= begin && search_address < end {
                found_address = true;
                break;
            }
        }

        if found_address {
            debug_info_offset = Some(header.debug_info_offset());
            break;
        }
    }

    let debug_info_offset = match debug_info_offset {
        Some(offset) => offset,
        None => return Err(anyhow!("can not find arange")),
    };

    // get debug info
    // catch header
    let debug_info_header = dwarf.debug_info.header_from_offset(debug_info_offset)?;

    if verbose {
        println!("got debug info header 0x{:016x}", debug_info_offset.0);
    };

    let debug_info_unit = dwarf.unit(debug_info_header)?;
    let mut debug_info_entries = debug_info_unit.entries();

    let mut found_symbol_name: Option<String> = None;
    while let Some(_) = debug_info_entries.next_entry()? {
        if let Some(entry) = debug_info_entries.current() {
            if entry.tag() == DW_TAG_subprogram {
                let mut low_pc: Option<u64> = None;
                let mut high_pc: Option<u64> = None;
                if let Ok(Some(gimli::AttributeValue::Addr(lowpc_val))) =
                    entry.attr_value(gimli::DW_AT_low_pc)
                {
                    low_pc = Some(lowpc_val);
                    let high_pc_value = entry.attr_value(gimli::DW_AT_high_pc);

                    if verbose {
                        println!("high pc value : {:?}", high_pc_value);
                    }

                    if let Ok(Some(high_pc_data)) = high_pc_value {
                        if let gimli::AttributeValue::Addr(addr) = high_pc_data {
                            high_pc = Some(addr);
                        } else if let gimli::AttributeValue::Udata(size) = high_pc_data {
                            high_pc = Some(lowpc_val + size);
                        }
                    }
                }

                if verbose {
                    println!("low pc = {:?}, high pc = {:?}", low_pc, high_pc)
                }

                if let (Some(low_pc), Some(high_pc)) = (low_pc, high_pc) {
                    if search_address >= low_pc && search_address < high_pc {
                        if let Ok(Some(name)) = entry.attr_value(gimli::DW_AT_name) {
                            if let Ok(symbol_name) = dwarf.attr_string(&debug_info_unit, name) {
                                found_symbol_name = Some(symbol_name.to_string_lossy().to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    if verbose {
        println!("found_symbol_name = {:?}", found_symbol_name);
    };

    let mut found_file_name: Option<String> = None;
    let mut found_line: Option<u64> = None;
    if let Some(program) = debug_info_unit.line_program.clone() {
        let mut rows = program.rows();
        let mut last_file_name: Option<String> = None;
        let mut last_line: Option<u64> = None;
        while let Some((header, row)) = rows.next_row()? {
            if row.end_sequence() {
                if search_address < row.address() {
                    // got last filename and line
                    found_file_name = last_file_name.clone();
                    found_line = last_line;
                    if let Some(line_no) = found_line {
                        if line_no > 0 {
                            break;
                        }
                    }
                }
                continue;
            }
            if let Some(file) = row.file(header) {
                let filename = dwarf
                    .attr_string(&debug_info_unit, file.path_name())?
                    .to_string_lossy();
                last_file_name = Some(filename.into_owned());
            }
            let line = match row.line() {
                Some(line) => line.get(),
                None => 0,
            };

            if search_address < row.address() {
                // got last filename and line
                found_file_name = last_file_name.clone();
                found_line = last_line;
                if let Some(line_no) = found_line {
                    if line_no > 0 {
                        break;
                    }
                }
            }
            last_line = Some(line);
        }
    }

    if verbose {
        println!(
            "found_file_name = {:?} found_line = {:?}",
            found_file_name, found_line
        );
    };

    if let (Some(symbol_name), Some(file_name), Some(line)) =
        (found_symbol_name, found_file_name, found_line)
    {
        // expect format
        // main (in BinaryName) (main.m:100)

        let demangled_name = demangle::demangle_symbol(&symbol_name);
        let symbolize_result = format!(
            "{} (in {}) ({}:{})",
            demangled_name, object_filename, file_name, line
        );
        return Ok(symbolize_result);
    }
    Err(anyhow!("failed search symbol"))
}
