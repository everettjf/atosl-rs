//
// written by everettjf
// email : everettjf@live.com
// created at 2022-01-01
//
use crate::demangle;
use anyhow::{anyhow, Context, Result};
use gimli::{DW_TAG_subprogram, DebugInfoOffset, Dwarf, EndianSlice, RunTimeEndian};
use object::macho;
use object::read::macho::{FatArch, FatHeader};
use object::{Object, ObjectSection, ObjectSegment};
use std::path::Path;
use std::{borrow, fs};

pub fn print_addresses(
    object_path: &Path,
    load_address: u64,
    addresses: &[u64],
    verbose: bool,
    file_offset_type: bool,
    arch_filter: Option<&str>,
    uuid_filter: Option<&str>,
) -> Result<(), anyhow::Error> {
    let file = fs::File::open(object_path)
        .with_context(|| format!("failed to open object file: {}", object_path.display()))?;
    let mmap = unsafe { memmap::Mmap::map(&file)? };
    let object_filename = object_path
        .file_name()
        .ok_or_else(|| {
            anyhow!(
                "failed to get file name from path: {}",
                object_path.display()
            )
        })?
        .to_string_lossy()
        .to_string();

    let parsed_uuid_filter = uuid_filter.map(parse_uuid_string).transpose()?;
    let (object, selected_slice) =
        resolve_object_from_data(&mmap, arch_filter, parsed_uuid_filter, verbose)?;
    if verbose {
        if let Some(selected_slice) = selected_slice {
            println!("selected_slice: {selected_slice}");
        }
    }

    print_addresses_for_object(
        &object,
        &object_filename,
        load_address,
        addresses,
        verbose,
        file_offset_type,
    )
}

fn print_addresses_for_object(
    object: &object::File,
    object_filename: &str,
    load_address: u64,
    addresses: &[u64],
    verbose: bool,
    file_offset_type: bool,
) -> Result<(), anyhow::Error> {
    if is_object_dwarf(object) {
        if verbose {
            println!("resolver: dwarf");
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
            println!("resolver: symbol_table");
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

struct FatSlice<'data> {
    object: object::File<'data, &'data [u8]>,
    arch_name: String,
    uuid: Option<[u8; 16]>,
}

fn resolve_object_from_data<'data>(
    data: &'data [u8],
    arch_filter: Option<&str>,
    uuid_filter: Option<[u8; 16]>,
    verbose: bool,
) -> Result<(object::File<'data, &'data [u8]>, Option<String>), anyhow::Error> {
    let kind = object::FileKind::parse(data)?;
    match kind {
        object::FileKind::MachOFat32 => {
            let arches = FatHeader::parse_arch32(data)?;
            select_fat_slice(arches, data, arch_filter, uuid_filter, verbose)
        }
        object::FileKind::MachOFat64 => {
            let arches = FatHeader::parse_arch64(data)?;
            select_fat_slice(arches, data, arch_filter, uuid_filter, verbose)
        }
        _ => {
            let file = object::File::parse(data)?;
            validate_non_fat_filters(&file, arch_filter, uuid_filter)?;
            Ok((file, None))
        }
    }
}

fn validate_non_fat_filters(
    file: &object::File,
    arch_filter: Option<&str>,
    uuid_filter: Option<[u8; 16]>,
) -> Result<(), anyhow::Error> {
    if let Some(uuid_filter) = uuid_filter {
        let actual_uuid = file
            .mach_uuid()?
            .ok_or_else(|| anyhow!("--uuid was provided, but this file has no Mach-O UUID"))?;
        if actual_uuid != uuid_filter {
            return Err(anyhow!(
                "uuid mismatch: requested {}, actual {}",
                format_uuid(uuid_filter),
                format_uuid(actual_uuid)
            ));
        }
    }

    if let Some(arch_filter) = arch_filter {
        let architecture = file.architecture();
        let matches = architecture_matches_filter(architecture, arch_filter);
        if !matches {
            return Err(anyhow!(
                "architecture mismatch: requested '{}', actual '{:?}'",
                arch_filter,
                architecture
            ));
        }
    }
    Ok(())
}

fn select_fat_slice<'data, A: FatArch>(
    arches: &'data [A],
    data: &'data [u8],
    arch_filter: Option<&str>,
    uuid_filter: Option<[u8; 16]>,
    verbose: bool,
) -> Result<(object::File<'data, &'data [u8]>, Option<String>), anyhow::Error> {
    let mut slices = Vec::with_capacity(arches.len());
    for arch in arches {
        let cputype = arch.cputype();
        let cpusubtype = arch.cpusubtype();
        let arch_name = format_macho_arch_name(cputype, cpusubtype);
        let arch_data = arch.data(data)?;
        let object = object::File::parse(arch_data)?;
        let uuid = object.mach_uuid()?;
        slices.push(FatSlice {
            object,
            arch_name,
            uuid,
        });
    }

    if verbose {
        for slice in &slices {
            let uuid_str = slice
                .uuid
                .map(format_uuid)
                .unwrap_or_else(|| "-".to_string());
            println!("fat_slice: arch={} uuid={}", slice.arch_name, uuid_str);
        }
    }

    if slices.is_empty() {
        return Err(anyhow!("fat Mach-O has no slices"));
    }

    if arch_filter.is_none() && uuid_filter.is_none() {
        if slices.len() == 1 {
            let slice = slices.into_iter().next().expect("slice len checked");
            let selected = Some(format_selected_slice(&slice.arch_name, slice.uuid));
            return Ok((slice.object, selected));
        }
        return Err(anyhow!(
            "fat Mach-O contains multiple slices.\nUse -a/--arch or --uuid to select one.\nAvailable slices:\n{}",
            format_available_slices(&slices)
        ));
    }

    let available_slices = format_available_slices(&slices);
    let mut matches = slices
        .into_iter()
        .filter(|slice| {
            let arch_ok = arch_filter
                .map(|filter| macho_arch_matches_filter(&slice.arch_name, filter))
                .unwrap_or(true);
            let uuid_ok = uuid_filter
                .map(|uuid| slice.uuid == Some(uuid))
                .unwrap_or(true);
            arch_ok && uuid_ok
        })
        .collect::<Vec<_>>();

    match matches.len() {
        0 => Err(anyhow!(
            "no fat Mach-O slice matched arch={:?} uuid={:?}.\nAvailable slices:\n{}",
            arch_filter,
            uuid_filter.map(format_uuid),
            available_slices
        )),
        1 => {
            let slice = matches.pop().expect("slice len checked");
            let selected = Some(format_selected_slice(&slice.arch_name, slice.uuid));
            Ok((slice.object, selected))
        }
        _ => {
            let available = matches
                .iter()
                .map(|slice| format_selected_slice(&slice.arch_name, slice.uuid))
                .collect::<Vec<_>>()
                .join("\n");
            Err(anyhow!(
                "filters are ambiguous and matched multiple slices:\n{}",
                available
            ))
        }
    }
}

fn format_available_slices(slices: &[FatSlice]) -> String {
    slices
        .iter()
        .map(|slice| format_selected_slice(&slice.arch_name, slice.uuid))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_selected_slice(arch_name: &str, uuid: Option<[u8; 16]>) -> String {
    let uuid = uuid.map(format_uuid).unwrap_or_else(|| "-".to_string());
    format!("- arch={arch_name} uuid={uuid}")
}

fn parse_uuid_string(value: &str) -> Result<[u8; 16], anyhow::Error> {
    let hex = value
        .chars()
        .filter(|c| *c != '-')
        .collect::<String>()
        .to_ascii_lowercase();
    if hex.len() != 32 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!(
            "invalid uuid format: '{value}', expected 32 hex chars (with or without '-')"
        ));
    }

    let mut out = [0u8; 16];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let byte_str = std::str::from_utf8(chunk)?;
        out[i] = u8::from_str_radix(byte_str, 16)?;
    }
    Ok(out)
}

fn format_uuid(uuid: [u8; 16]) -> String {
    let hex = uuid
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<String>();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

fn normalize_arch(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>()
}

fn architecture_matches_filter(architecture: object::Architecture, arch_filter: &str) -> bool {
    let filter = normalize_arch(arch_filter);
    match filter.as_str() {
        "arm" | "armv7" | "armv7s" | "armv7k" => architecture == object::Architecture::Arm,
        "arm64" | "arm64e" | "aarch64" => architecture == object::Architecture::Aarch64,
        "x8664" | "x64" | "amd64" | "x8664h" => architecture == object::Architecture::X86_64,
        "x86" | "i386" => architecture == object::Architecture::I386,
        _ => false,
    }
}

fn macho_arch_matches_filter(arch_name: &str, arch_filter: &str) -> bool {
    let actual = normalize_arch(arch_name);
    let filter = normalize_arch(arch_filter);
    if actual == filter {
        return true;
    }
    matches!(
        (actual.as_str(), filter.as_str()),
        ("arm64", "aarch64")
            | ("aarch64", "arm64")
            | ("x8664", "amd64")
            | ("x8664", "x64")
            | ("i386", "x86")
    )
}

fn format_macho_arch_name(cputype: u32, cpusubtype: u32) -> String {
    let cpusubtype = cpusubtype & !macho::CPU_SUBTYPE_MASK;
    match cputype {
        macho::CPU_TYPE_ARM64 => match cpusubtype {
            macho::CPU_SUBTYPE_ARM64E => "arm64e".to_string(),
            _ => "arm64".to_string(),
        },
        macho::CPU_TYPE_ARM => match cpusubtype {
            macho::CPU_SUBTYPE_ARM_V7 => "armv7".to_string(),
            macho::CPU_SUBTYPE_ARM_V7F => "armv7f".to_string(),
            macho::CPU_SUBTYPE_ARM_V7S => "armv7s".to_string(),
            macho::CPU_SUBTYPE_ARM_V7K => "armv7k".to_string(),
            macho::CPU_SUBTYPE_ARM_V8 => "armv8".to_string(),
            _ => "arm".to_string(),
        },
        macho::CPU_TYPE_X86_64 => match cpusubtype {
            macho::CPU_SUBTYPE_X86_64_H => "x86_64h".to_string(),
            _ => "x86_64".to_string(),
        },
        macho::CPU_TYPE_X86 => "i386".to_string(),
        _ => format!("cputype{}_subtype{}", cputype, cpusubtype),
    }
}

fn is_object_dwarf(object: &object::File) -> bool {
    object.section_by_name("__debug_line").is_some()
}

fn find_text_vmaddr(object: &object::File) -> Result<u64, anyhow::Error> {
    for segment in object.segments() {
        if let Some(name) = segment.name()? {
            if name == "__TEXT" {
                return Ok(segment.address());
            }
        }
    }
    Ok(0)
}

fn calculate_search_address(
    load_address: u64,
    address: u64,
    text_vmaddr: u64,
    file_offset_type: bool,
) -> Result<u64, anyhow::Error> {
    let base = address
        .checked_sub(load_address)
        .ok_or_else(|| anyhow!("address is smaller than load address"))?;

    if file_offset_type {
        Ok(base)
    } else {
        base.checked_add(text_vmaddr)
            .ok_or_else(|| anyhow!("address overflow while applying __TEXT vmaddr"))
    }
}

fn symbol_symbolize_addresses(
    object: &object::File,
    object_filename: &str,
    load_address: u64,
    addresses: &[u64],
    verbose: bool,
    file_offset_type: bool,
) -> Result<(), anyhow::Error> {
    let text_vmaddr = find_text_vmaddr(object)?;

    for &address in addresses {
        if verbose {
            println!("---------------------------------------------");
            println!("BEGIN ADDRESS {} | {:016x}", address, address);
        }

        let symbol_result = symbol_symbolize_address(
            object,
            object_filename,
            load_address,
            address,
            text_vmaddr,
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
    file_offset_type: bool,
) -> Result<String, anyhow::Error> {
    let search_address =
        calculate_search_address(load_address, address, text_vmaddr, file_offset_type)?;

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
    addresses: &[u64],
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
    let dwarf = dwarf_cow.borrow(|section| gimli::EndianSlice::new(section.as_ref(), endian));

    let text_vmaddr = find_text_vmaddr(object)?;

    for &address in addresses {
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
                    object,
                    object_filename,
                    load_address,
                    address,
                    text_vmaddr,
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
    let search_address =
        calculate_search_address(load_address, address, text_vmaddr, file_offset_type)?;

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
    while debug_info_entries.next_entry()?.is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uuid_accepts_hyphenated_and_plain() {
        let a = parse_uuid_string("29118F18-9DFC-36A8-9028-A19B13996D5E").unwrap();
        let b = parse_uuid_string("29118f189dfc36a89028a19b13996d5e").unwrap();
        assert_eq!(a, b);
        assert_eq!(format_uuid(a), "29118F18-9DFC-36A8-9028-A19B13996D5E");
    }

    #[test]
    fn parse_uuid_rejects_invalid() {
        assert!(parse_uuid_string("not-a-uuid").is_err());
        assert!(parse_uuid_string("1234").is_err());
    }

    #[test]
    fn macho_arch_aliases_match() {
        assert!(macho_arch_matches_filter("arm64", "aarch64"));
        assert!(macho_arch_matches_filter("x86_64", "amd64"));
        assert!(macho_arch_matches_filter("i386", "x86"));
    }
}
