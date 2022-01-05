//
// written by everettjf
// email : everettjf@live.com
// created at 2022-01-01
//
use crate::demangle;
use anyhow::{anyhow, Result};
use gimli::{DW_TAG_subprogram, DebugInfoOffset, Dwarf, EndianSlice, RunTimeEndian};
use object::{Object, ObjectSection, ObjectSegment};
use std::path::Path;
use std::{borrow, fs};

pub fn print_addresses(
    object_path: &str,
    load_address: u64,
    addresses: Vec<u64>,
    debug_mode: bool,
    file_offset_type: bool,
) -> Result<(), anyhow::Error> {
    let file = fs::File::open(&object_path)?;
    let mmap = unsafe { memmap::Mmap::map(&file)? };
    let object = object::File::parse(&*mmap)?;
    let object_filename = Path::new(&object_path)
        .file_name()
        .ok_or(anyhow!("file name error"))?
        .to_str()
        .ok_or(anyhow!("file name error(to_str)"))?;

    if is_object_dwarf(&object) {
        if debug_mode {
            println!("dwarf");
        }
        dwarf_symbolize_addresses(
            &object,
            object_filename,
            load_address,
            addresses,
            debug_mode,
            file_offset_type,
        )
    } else {
        if debug_mode {
            println!("symbols");
        }
        symbol_symbolize_addresses(
            &object,
            object_filename,
            load_address,
            addresses,
            debug_mode,
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
    debug_mode: bool,
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
        if debug_mode {
            println!("---------------------------------------------");
            println!("BEGIN ADDRESS {} | {:016x}", address, address);
        }

        let symbol_result = symbol_symbolize_address(
            &object,
            object_filename,
            load_address,
            address,
            text_vmaddr,
            debug_mode,
            file_offset_type,
        );
        if debug_mode {
            println!("RESULT:")
        }
        match symbol_result {
            Ok(symbol) => println!("{}", symbol),
            Err(err) => println!("N/A - {}", err),
        };

        if debug_mode {
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
    _debug_mode: bool,
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
    debug_mode: bool,
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
        if debug_mode {
            println!("---------------------------------------------");
            println!("BEGIN ADDRESS {} | {:016x}", address, address);
        }

        let symbol_result = dwarf_symbolize_address(
            &dwarf,
            object_filename,
            load_address,
            address,
            text_vmaddr,
            debug_mode,
            file_offset_type,
        );
        if debug_mode {
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
                    debug_mode,
                    file_offset_type,
                );
                match symbol_result {
                    Ok(symbol) => println!("{}", symbol),
                    Err(err) => println!("N/A - {}", err),
                };
            }
        };

        if debug_mode {
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
    debug_mode: bool,
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

    if debug_mode {
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

                    if debug_mode {
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

                if debug_mode {
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
    if debug_mode {
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
                    break;
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
                break;
            }
            last_line = Some(line);
        }
    }

    if debug_mode {
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
