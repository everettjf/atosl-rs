use crate::demangle;
use anyhow::{anyhow, Context, Result};
use gimli::{Dwarf, EndianSlice, RunTimeEndian, Unit};
use object::macho;
use object::read::macho::{FatArch, FatHeader};
use object::{Object, ObjectSection, ObjectSegment};
use serde::Serialize;
use std::borrow;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Text,
    Json,
    JsonPretty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolverKind {
    Dwarf,
    SymbolTable,
}

#[derive(Clone, Debug)]
pub struct SymbolizeOptions {
    pub object_path: PathBuf,
    pub load_address: u64,
    pub addresses: Vec<u64>,
    pub verbose: bool,
    pub file_offsets: bool,
    pub arch: Option<String>,
    pub uuid: Option<String>,
    pub format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SelectedSlice {
    pub arch: String,
    pub uuid: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SymbolizedFrame {
    pub requested_address: u64,
    pub lookup_address: u64,
    pub symbol: String,
    pub object_name: String,
    pub offset: u64,
    pub resolver: ResolverKind,
    pub location: Option<SourceLocation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SymbolizeOutcome {
    Resolved(SymbolizedFrame),
    Unresolved {
        requested_address: u64,
        error: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SymbolizeReport {
    pub object_path: String,
    pub object_name: String,
    pub selected_slice: Option<SelectedSlice>,
    pub frames: Vec<SymbolizeOutcome>,
}

struct FatSlice<'data> {
    object: object::File<'data, &'data [u8]>,
    selected_slice: SelectedSlice,
}

struct ResolvedObject<'data> {
    object: object::File<'data, &'data [u8]>,
    object_name: String,
    selected_slice: Option<SelectedSlice>,
}

pub fn run(options: SymbolizeOptions) -> Result<i32> {
    let report = symbolize_path(&options)?;
    emit_report(&report, options.format, options.verbose);
    Ok(0)
}

pub fn symbolize_path(options: &SymbolizeOptions) -> Result<SymbolizeReport> {
    let file = fs::File::open(&options.object_path).with_context(|| {
        format!(
            "failed to open object file: {}",
            options.object_path.display()
        )
    })?;
    let mmap = unsafe { memmap2::Mmap::map(&file) }
        .with_context(|| format!("failed to memory-map: {}", options.object_path.display()))?;

    let parsed_uuid_filter = options.uuid.as_deref().map(parse_uuid_string).transpose()?;
    let resolved = resolve_object_from_data(
        &mmap,
        &options.object_path,
        options.arch.as_deref(),
        parsed_uuid_filter,
        options.verbose,
    )?;

    let endian = if resolved.object.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };
    let text_vmaddr = find_text_vmaddr(&resolved.object)?;
    let frames = if is_object_dwarf(&resolved.object) {
        let dwarf_cow = Dwarf::load(
            |section_id| -> Result<borrow::Cow<'_, [u8]>, gimli::Error> {
                let macho_name = section_id
                    .name()
                    .strip_prefix('.')
                    .map(|name| format!("__{name}"));

                match resolved
                    .object
                    .section_by_name(section_id.name())
                    .or_else(|| {
                        macho_name
                            .as_deref()
                            .and_then(|name| resolved.object.section_by_name(name))
                    }) {
                    Some(section) => Ok(section
                        .uncompressed_data()
                        .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
                    None => Ok(borrow::Cow::Borrowed(&[][..])),
                }
            },
        )?;
        let dwarf = dwarf_cow.borrow(|section| EndianSlice::new(section.as_ref(), endian));

        options
            .addresses
            .iter()
            .copied()
            .map(|requested_address| {
                symbolize_address(
                    &resolved.object,
                    &resolved.object_name,
                    Some(&dwarf),
                    options.load_address,
                    requested_address,
                    text_vmaddr,
                    options.file_offsets,
                )
            })
            .collect()
    } else {
        options
            .addresses
            .iter()
            .copied()
            .map(|requested_address| {
                symbolize_address(
                    &resolved.object,
                    &resolved.object_name,
                    None,
                    options.load_address,
                    requested_address,
                    text_vmaddr,
                    options.file_offsets,
                )
            })
            .collect()
    };

    Ok(SymbolizeReport {
        object_path: options.object_path.display().to_string(),
        object_name: resolved.object_name,
        selected_slice: resolved.selected_slice,
        frames,
    })
}

fn emit_report(report: &SymbolizeReport, format: OutputFormat, verbose: bool) {
    match format {
        OutputFormat::Text => emit_text_report(report, verbose),
        OutputFormat::Json => println!("{}", serde_json::to_string(report).unwrap()),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(report).unwrap()),
    }
}

fn emit_text_report(report: &SymbolizeReport, verbose: bool) {
    if verbose {
        eprintln!("object: {}", report.object_path);
        if let Some(selected_slice) = &report.selected_slice {
            eprintln!(
                "selected_slice: arch={} uuid={}",
                selected_slice.arch,
                selected_slice.uuid.as_deref().unwrap_or("-")
            );
        }
    }

    for frame in &report.frames {
        match frame {
            SymbolizeOutcome::Resolved(frame) => {
                if verbose {
                    eprintln!(
                        "frame: requested=0x{requested:016x} lookup=0x{lookup:016x} resolver={resolver:?} status=resolved",
                        requested = frame.requested_address,
                        lookup = frame.lookup_address,
                        resolver = frame.resolver,
                    );
                }
                println!("{}", format_text_frame(frame));
            }
            SymbolizeOutcome::Unresolved {
                requested_address,
                error,
            } => {
                if verbose {
                    eprintln!(
                        "frame: requested=0x{requested_address:016x} status=unresolved error={error}"
                    );
                }
                println!("N/A - {error}");
            }
        }
    }
}

fn format_text_frame(frame: &SymbolizedFrame) -> String {
    match &frame.location {
        Some(location) => format!(
            "{} (in {}) ({}:{})",
            frame.symbol, frame.object_name, location.file, location.line
        ),
        None => format!(
            "{} (in {}) + {}",
            frame.symbol, frame.object_name, frame.offset
        ),
    }
}

fn symbolize_address<'data>(
    object: &object::File<'data, &'data [u8]>,
    object_name: &str,
    dwarf: Option<&Dwarf<EndianSlice<'data, RunTimeEndian>>>,
    load_address: u64,
    requested_address: u64,
    text_vmaddr: u64,
    file_offsets: bool,
) -> SymbolizeOutcome {
    let search_address = match calculate_search_address(
        load_address,
        requested_address,
        text_vmaddr,
        file_offsets,
    ) {
        Ok(search_address) => search_address,
        Err(err) => {
            return SymbolizeOutcome::Unresolved {
                requested_address,
                error: err.to_string(),
            };
        }
    };

    if let Some(dwarf) = dwarf {
        if let Ok(frame) =
            dwarf_symbolize_address(dwarf, object_name, requested_address, search_address)
        {
            return SymbolizeOutcome::Resolved(frame);
        }
    }

    match symbol_symbolize_address(object, object_name, requested_address, search_address) {
        Ok(frame) => SymbolizeOutcome::Resolved(frame),
        Err(err) => SymbolizeOutcome::Unresolved {
            requested_address,
            error: err.to_string(),
        },
    }
}

fn resolve_object_from_data<'data>(
    data: &'data [u8],
    object_path: &Path,
    arch_filter: Option<&str>,
    uuid_filter: Option<[u8; 16]>,
    verbose: bool,
) -> Result<ResolvedObject<'data>> {
    let kind = object::FileKind::parse(data)?;
    let object_name = object_path
        .file_name()
        .ok_or_else(|| {
            anyhow!(
                "failed to derive object filename from {}",
                object_path.display()
            )
        })?
        .to_string_lossy()
        .to_string();

    match kind {
        object::FileKind::MachOFat32 => {
            let arches = FatHeader::parse_arch32(data)?;
            let selected = select_fat_slice(arches, data, arch_filter, uuid_filter, verbose)?;
            Ok(ResolvedObject {
                object: selected.object,
                object_name,
                selected_slice: Some(selected.selected_slice),
            })
        }
        object::FileKind::MachOFat64 => {
            let arches = FatHeader::parse_arch64(data)?;
            let selected = select_fat_slice(arches, data, arch_filter, uuid_filter, verbose)?;
            Ok(ResolvedObject {
                object: selected.object,
                object_name,
                selected_slice: Some(selected.selected_slice),
            })
        }
        _ => {
            let file = object::File::parse(data)?;
            validate_non_fat_filters(&file, arch_filter, uuid_filter)?;
            let selected_slice = file.mach_uuid()?.map(|uuid| SelectedSlice {
                arch: format!("{:?}", file.architecture()).to_lowercase(),
                uuid: Some(format_uuid(uuid)),
            });
            Ok(ResolvedObject {
                object: file,
                object_name,
                selected_slice,
            })
        }
    }
}

fn validate_non_fat_filters<'data>(
    file: &object::File<'data, &'data [u8]>,
    arch_filter: Option<&str>,
    uuid_filter: Option<[u8; 16]>,
) -> Result<()> {
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
        if !architecture_matches_filter(architecture, arch_filter) {
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
) -> Result<FatSlice<'data>> {
    let mut slices = Vec::with_capacity(arches.len());
    for arch in arches {
        let arch_name = format_macho_arch_name(arch.cputype(), arch.cpusubtype());
        let object = object::File::parse(arch.data(data)?)?;
        let uuid = object.mach_uuid()?.map(format_uuid);
        slices.push(FatSlice {
            object,
            selected_slice: SelectedSlice {
                arch: arch_name,
                uuid,
            },
        });
    }

    if verbose {
        for slice in &slices {
            eprintln!(
                "fat_slice: arch={} uuid={}",
                slice.selected_slice.arch,
                slice.selected_slice.uuid.as_deref().unwrap_or("-")
            );
        }
    }

    if slices.is_empty() {
        return Err(anyhow!("fat Mach-O has no slices"));
    }

    if arch_filter.is_none() && uuid_filter.is_none() {
        return match slices.len() {
            1 => Ok(slices.remove(0)),
            _ => Err(anyhow!(
                "fat Mach-O contains multiple slices.\nUse -a/--arch or --uuid to select one.\nAvailable slices:\n{}",
                format_available_slices(&slices)
            )),
        };
    }

    let available_slices = format_available_slices(&slices);
    let mut matches = slices
        .into_iter()
        .filter(|slice| {
            let arch_ok = arch_filter
                .map(|filter| macho_arch_matches_filter(&slice.selected_slice.arch, filter))
                .unwrap_or(true);
            let uuid_ok = uuid_filter
                .map(|uuid| {
                    slice.selected_slice.uuid.as_deref() == Some(format_uuid(uuid).as_str())
                })
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
        1 => Ok(matches.pop().expect("match length checked")),
        _ => Err(anyhow!(
            "filters are ambiguous and matched multiple slices:\n{}",
            format_available_slices(&matches)
        )),
    }
}

fn format_available_slices(slices: &[FatSlice<'_>]) -> String {
    slices
        .iter()
        .map(|slice| {
            format!(
                "- arch={} uuid={}",
                slice.selected_slice.arch,
                slice.selected_slice.uuid.as_deref().unwrap_or("-")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_uuid_string(value: &str) -> Result<[u8; 16]> {
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
    for (index, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let byte_str = std::str::from_utf8(chunk)?;
        out[index] = u8::from_str_radix(byte_str, 16)?;
    }
    Ok(out)
}

fn format_uuid(uuid: [u8; 16]) -> String {
    let hex = uuid
        .iter()
        .map(|byte| format!("{byte:02X}"))
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
        .filter(|character| character.is_ascii_alphanumeric())
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
        _ => format!("cputype{cputype}_subtype{cpusubtype}"),
    }
}

fn is_object_dwarf<'data>(object: &object::File<'data, &'data [u8]>) -> bool {
    object.section_by_name("__debug_line").is_some()
        || object.section_by_name(".debug_line").is_some()
}

fn find_text_vmaddr<'data>(object: &object::File<'data, &'data [u8]>) -> Result<u64> {
    for segment in object.segments() {
        if let Some(name) = segment.name()? {
            if name == "__TEXT" {
                return Ok(segment.address());
            }
        }
    }

    if let Some(section) = object
        .section_by_name(".text")
        .or_else(|| object.section_by_name("__text"))
    {
        return Ok(section.address());
    }

    Ok(0)
}

fn calculate_search_address(
    load_address: u64,
    address: u64,
    text_vmaddr: u64,
    file_offset_type: bool,
) -> Result<u64> {
    let base = address
        .checked_sub(load_address)
        .ok_or_else(|| anyhow!("address is smaller than load address"))?;

    if file_offset_type {
        Ok(base)
    } else {
        base.checked_add(text_vmaddr)
            .ok_or_else(|| anyhow!("address overflow while applying text vmaddr"))
    }
}

fn symbol_symbolize_address<'data>(
    object: &object::File<'data, &'data [u8]>,
    object_name: &str,
    requested_address: u64,
    search_address: u64,
) -> Result<SymbolizedFrame> {
    let symbols = object.symbol_map();
    let found_symbol = symbols
        .get(search_address)
        .ok_or_else(|| anyhow!("failed to search symbol table"))?;
    let offset = search_address.saturating_sub(found_symbol.address());

    Ok(SymbolizedFrame {
        requested_address,
        lookup_address: search_address,
        symbol: demangle::demangle_symbol(found_symbol.name()),
        object_name: object_name.to_string(),
        offset,
        resolver: ResolverKind::SymbolTable,
        location: None,
    })
}

fn dwarf_symbolize_address(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    object_name: &str,
    requested_address: u64,
    search_address: u64,
) -> Result<SymbolizedFrame> {
    if let Some(frame) =
        dwarf_symbolize_in_aranges(dwarf, object_name, requested_address, search_address)?
    {
        return Ok(frame);
    }

    let mut units = dwarf.units();
    while let Some(header) = units.next()? {
        let unit = dwarf.unit(header)?;
        if let Some(frame) =
            dwarf_symbolize_in_unit(dwarf, &unit, object_name, requested_address, search_address)?
        {
            return Ok(frame);
        }
    }

    Err(anyhow!("failed to search DWARF"))
}

fn dwarf_symbolize_in_aranges(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    object_name: &str,
    requested_address: u64,
    search_address: u64,
) -> Result<Option<SymbolizedFrame>> {
    let mut headers = dwarf.debug_aranges.headers();
    while let Some(header) = headers.next()? {
        let mut entries = header.entries();
        while let Some(entry) = entries.next()? {
            let end = entry
                .address()
                .checked_add(entry.length())
                .ok_or_else(|| anyhow!("DWARF address range overflow"))?;
            if (entry.address()..end).contains(&search_address) {
                let unit = dwarf.unit(
                    dwarf
                        .debug_info
                        .header_from_offset(header.debug_info_offset())?,
                )?;
                return dwarf_symbolize_in_unit(
                    dwarf,
                    &unit,
                    object_name,
                    requested_address,
                    search_address,
                );
            }
        }
    }

    Ok(None)
}

fn dwarf_symbolize_in_unit(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    unit: &Unit<EndianSlice<'_, RunTimeEndian>>,
    object_name: &str,
    requested_address: u64,
    search_address: u64,
) -> Result<Option<SymbolizedFrame>> {
    let Some(subprogram) = find_subprogram_in_unit(dwarf, unit, search_address)? else {
        return Ok(None);
    };
    let location = match subprogram.location {
        Some(location) => Some(location),
        None => find_source_location_in_unit(dwarf, unit, search_address)?,
    };

    let offset = search_address.saturating_sub(subprogram.low_pc);

    match location {
        Some(location) => Ok(Some(SymbolizedFrame {
            requested_address,
            lookup_address: search_address,
            symbol: demangle::demangle_symbol(&subprogram.symbol_name),
            object_name: object_name.to_string(),
            offset,
            resolver: ResolverKind::Dwarf,
            location: Some(location),
        })),
        None => Ok(Some(SymbolizedFrame {
            requested_address,
            lookup_address: search_address,
            symbol: demangle::demangle_symbol(&subprogram.symbol_name),
            object_name: object_name.to_string(),
            offset,
            resolver: ResolverKind::Dwarf,
            location: None,
        })),
    }
}

struct MatchedSubprogram {
    symbol_name: String,
    low_pc: u64,
    location: Option<SourceLocation>,
}

fn find_subprogram_in_unit(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    unit: &Unit<EndianSlice<'_, RunTimeEndian>>,
    search_address: u64,
) -> Result<Option<MatchedSubprogram>> {
    let mut entries = unit.entries();
    while entries.next_entry()?.is_some() {
        let Some(entry) = entries.current() else {
            continue;
        };

        if entry.tag() != gimli::DW_TAG_subprogram {
            continue;
        }

        let Some((low_pc, high_pc)) = subprogram_range(dwarf, unit, entry)? else {
            continue;
        };

        if !(low_pc..high_pc).contains(&search_address) {
            continue;
        }

        for attr in [
            gimli::DW_AT_linkage_name,
            gimli::DW_AT_MIPS_linkage_name,
            gimli::DW_AT_name,
        ] {
            if let Ok(Some(value)) = entry.attr_value(attr) {
                if let Ok(name) = dwarf.attr_string(unit, value) {
                    return Ok(Some(MatchedSubprogram {
                        symbol_name: name.to_string_lossy().into_owned(),
                        low_pc,
                        location: find_decl_location(dwarf, unit, entry)?,
                    }));
                }
            }
        }
    }

    Ok(None)
}

fn find_decl_location(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    unit: &Unit<EndianSlice<'_, RunTimeEndian>>,
    entry: &gimli::DebuggingInformationEntry<'_, '_, EndianSlice<'_, RunTimeEndian>>,
) -> Result<Option<SourceLocation>> {
    let Some(program) = unit.line_program.as_ref() else {
        return Ok(None);
    };

    let file_index = match entry.attr_value(gimli::DW_AT_decl_file)? {
        Some(gimli::AttributeValue::FileIndex(index)) => index,
        _ => return Ok(None),
    };
    let line = match entry.attr_value(gimli::DW_AT_decl_line)? {
        Some(gimli::AttributeValue::Udata(line)) => line,
        _ => return Ok(None),
    };

    let Some(file) = program.header().file(file_index) else {
        return Ok(None);
    };

    Ok(Some(SourceLocation {
        file: resolve_line_file(dwarf, unit, program.header(), file)?,
        line,
    }))
}

fn subprogram_range(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    unit: &Unit<EndianSlice<'_, RunTimeEndian>>,
    entry: &gimli::DebuggingInformationEntry<'_, '_, EndianSlice<'_, RunTimeEndian>>,
) -> Result<Option<(u64, u64)>> {
    let low_pc = match entry.attr_value(gimli::DW_AT_low_pc)? {
        Some(value) => resolve_attr_address(dwarf, unit, value)?,
        _ => return Ok(None),
    };
    let Some(low_pc) = low_pc else {
        return Ok(None);
    };

    let high_pc = match entry.attr_value(gimli::DW_AT_high_pc)? {
        Some(gimli::AttributeValue::Addr(value)) => value,
        Some(gimli::AttributeValue::Udata(size)) => low_pc
            .checked_add(size)
            .ok_or_else(|| anyhow!("DWARF high_pc overflow"))?,
        Some(value) => match resolve_attr_address(dwarf, unit, value)? {
            Some(address) => address,
            None => return Ok(None),
        },
        _ => return Ok(None),
    };

    Ok(Some((low_pc, high_pc)))
}

fn resolve_attr_address(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    unit: &Unit<EndianSlice<'_, RunTimeEndian>>,
    value: gimli::AttributeValue<EndianSlice<'_, RunTimeEndian>>,
) -> Result<Option<u64>> {
    match value {
        gimli::AttributeValue::Addr(address) => Ok(Some(address)),
        other => Ok(dwarf.attr_address(unit, other)?),
    }
}

fn find_source_location_in_unit(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    unit: &Unit<EndianSlice<'_, RunTimeEndian>>,
    search_address: u64,
) -> Result<Option<SourceLocation>> {
    let Some(program) = unit.line_program.clone() else {
        return Ok(None);
    };

    let mut rows = program.rows();
    let mut last_file_name: Option<String> = None;
    let mut best_match: Option<(u64, String, u64)> = None;

    while let Some((header, row)) = rows.next_row()? {
        if row.end_sequence() {
            continue;
        }

        if let Some(file) = row.file(header) {
            last_file_name = Some(resolve_line_file(dwarf, unit, header, file)?);
        }

        let Some(file_name) = last_file_name.clone() else {
            continue;
        };

        let line = row.line().map(|line| line.get()).unwrap_or(0);
        let row_address = row.address();

        if row_address > search_address {
            break;
        }

        best_match = Some((row_address, file_name, line));
    }

    Ok(best_match.and_then(|(_, file, line)| {
        if line == 0 {
            None
        } else {
            Some(SourceLocation { file, line })
        }
    }))
}

fn resolve_line_file(
    dwarf: &Dwarf<EndianSlice<'_, RunTimeEndian>>,
    unit: &Unit<EndianSlice<'_, RunTimeEndian>>,
    header: &gimli::LineProgramHeader<EndianSlice<'_, RunTimeEndian>>,
    file: &gimli::FileEntry<EndianSlice<'_, RunTimeEndian>>,
) -> Result<String> {
    let file_name = dwarf
        .attr_string(unit, file.path_name())?
        .to_string_lossy()
        .into_owned();

    let directory = file
        .directory(header)
        .and_then(|directory| dwarf.attr_string(unit, directory).ok())
        .map(|directory| directory.to_string_lossy().into_owned());

    Ok(match directory {
        Some(directory) if !directory.is_empty() => join_debug_path(&directory, &file_name),
        _ => file_name,
    })
}

fn join_debug_path(directory: &str, file_name: &str) -> String {
    if directory == "." {
        if file_name.starts_with("./") {
            return file_name.to_string();
        }
        return format!("./{file_name}");
    }

    if file_name.starts_with(directory) {
        return file_name.to_string();
    }

    format!("{directory}/{file_name}")
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

    #[test]
    fn calculate_search_address_for_file_offsets() {
        assert_eq!(
            calculate_search_address(0x1000, 0x1030, 0x2000, true).unwrap(),
            0x30
        );
    }

    #[test]
    fn calculate_search_address_for_vmaddr_mode() {
        assert_eq!(
            calculate_search_address(0x1000, 0x1030, 0x2000, false).unwrap(),
            0x2030
        );
    }

    #[test]
    fn format_text_frame_with_location() {
        let frame = SymbolizedFrame {
            requested_address: 1,
            lookup_address: 1,
            symbol: "demo".to_string(),
            object_name: "fixture".to_string(),
            offset: 0,
            resolver: ResolverKind::Dwarf,
            location: Some(SourceLocation {
                file: "src/main.rs".to_string(),
                line: 7,
            }),
        };

        assert_eq!(
            format_text_frame(&frame),
            "demo (in fixture) (src/main.rs:7)"
        );
    }
}
