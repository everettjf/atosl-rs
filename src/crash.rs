//! Whole-crash-report symbolication.
//!
//! Parses Apple crash reports (modern `.ips` JSON and legacy `.crash` text),
//! locates the matching binary/dSYM for each referenced image by UUID/build-id,
//! and rewrites the report with symbolicated frames.

use crate::atosl::{
    index_object_dirs, normalize_hex_id, with_symbolizer, OutputFormat, SymbolizeOptions,
    SymbolizeOutcome, SymbolizedFrame,
};
use anyhow::{anyhow, Context as _, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;

pub struct CrashSymbolizeOptions {
    /// Directories of dSYMs/binaries searched per image by UUID/build-id.
    pub dsym_dirs: Vec<PathBuf>,
    pub verbose: bool,
}

#[derive(Debug)]
struct Image {
    uuid_hex: Option<String>,
    base: u64,
}

/// Symbolicates a crash report, returning the rewritten report text.
pub fn symbolicate(input: &str, options: &CrashSymbolizeOptions) -> Result<String> {
    if input.trim_start().starts_with('{') {
        symbolicate_ips(input, options)
    } else {
        Ok(symbolicate_text(input, options))
    }
}

// Resolves each image's binary once, symbolizes all of its addresses in a single
// pass, and returns the outcome for every request keyed by the caller's key.
fn symbolize_grouped<K: Eq + Hash + Clone>(
    images: &[Image],
    requests: &[(K, usize, u64)],
    options: &CrashSymbolizeOptions,
) -> HashMap<K, SymbolizeOutcome> {
    let mut results = HashMap::new();
    if requests.is_empty() {
        return results;
    }

    let index = index_object_dirs(&options.dsym_dirs);

    let mut by_image: HashMap<usize, Vec<(K, u64)>> = HashMap::new();
    for (key, image_index, address) in requests {
        by_image
            .entry(*image_index)
            .or_default()
            .push((key.clone(), *address));
    }

    for (image_index, reqs) in by_image {
        let Some(image) = images.get(image_index) else {
            continue;
        };
        let Some(uuid_hex) = &image.uuid_hex else {
            continue;
        };
        let Some(object_path) = index.get(&normalize_hex_id(uuid_hex)) else {
            continue;
        };

        let opts = SymbolizeOptions {
            object_path: object_path.clone(),
            load_address: image.base,
            addresses: Vec::new(),
            verbose: options.verbose,
            file_offsets: false,
            arch: None,
            uuid: Some(uuid_hex.clone()),
            format: OutputFormat::Text,
            input: None,
            debug_dirs: options.dsym_dirs.clone(),
        };

        let addresses: Vec<u64> = reqs.iter().map(|(_, address)| *address).collect();
        let outcomes = with_symbolizer(&opts, |symbolizer, _, _| {
            addresses
                .iter()
                .map(|address| symbolizer.symbolize(image.base, *address, false))
                .collect::<Vec<_>>()
        });

        if let Ok(outcomes) = outcomes {
            for ((key, _), outcome) in reqs.into_iter().zip(outcomes) {
                results.insert(key, outcome);
            }
        }
    }

    results
}

fn symbolicate_ips(input: &str, options: &CrashSymbolizeOptions) -> Result<String> {
    let (header, body_str) = input
        .split_once('\n')
        .ok_or_else(|| anyhow!("ips report is missing its body after the header line"))?;
    let mut body: Value =
        serde_json::from_str(body_str.trim()).context("failed to parse ips report body as JSON")?;

    let images = parse_ips_images(&body)?;

    let mut requests: Vec<((usize, usize), usize, u64)> = Vec::new();
    if let Some(threads) = body.get("threads").and_then(Value::as_array) {
        for (t, thread) in threads.iter().enumerate() {
            let Some(frames) = thread.get("frames").and_then(Value::as_array) else {
                continue;
            };
            for (f, frame) in frames.iter().enumerate() {
                let (Some(idx), Some(offset)) = (
                    frame.get("imageIndex").and_then(Value::as_u64),
                    frame.get("imageOffset").and_then(Value::as_u64),
                ) else {
                    continue;
                };
                let idx = idx as usize;
                if let Some(image) = images.get(idx) {
                    requests.push(((t, f), idx, image.base.wrapping_add(offset)));
                }
            }
        }
    }

    let results = symbolize_grouped(&images, &requests, options);

    if let Some(threads) = body.get_mut("threads").and_then(Value::as_array_mut) {
        for (t, thread) in threads.iter_mut().enumerate() {
            let Some(frames) = thread.get_mut("frames").and_then(Value::as_array_mut) else {
                continue;
            };
            for (f, frame) in frames.iter_mut().enumerate() {
                if let Some(SymbolizeOutcome::Resolved(resolved)) = results.get(&(t, f)) {
                    apply_ips_frame(frame, resolved);
                }
            }
        }
    }

    let serialized = serde_json::to_string(&body).context("failed to serialize ips report")?;
    Ok(format!("{header}\n{serialized}"))
}

fn parse_ips_images(body: &Value) -> Result<Vec<Image>> {
    let images = body
        .get("usedImages")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("ips report has no usedImages array"))?;
    Ok(images
        .iter()
        .map(|image| Image {
            uuid_hex: image
                .get("uuid")
                .and_then(Value::as_str)
                .map(normalize_hex_id),
            base: image.get("base").and_then(Value::as_u64).unwrap_or(0),
        })
        .collect())
}

fn apply_ips_frame(frame: &mut Value, resolved: &SymbolizedFrame) {
    let Some(object) = frame.as_object_mut() else {
        return;
    };
    object.insert("symbol".to_string(), Value::String(resolved.symbol.clone()));
    object.insert("symbolLocation".to_string(), Value::from(resolved.offset));
    if let Some(location) = &resolved.location {
        object.insert(
            "sourceFile".to_string(),
            Value::String(location.file.clone()),
        );
        object.insert("sourceLine".to_string(), Value::from(location.line));
    }
}

fn symbolicate_text(input: &str, options: &CrashSymbolizeOptions) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let (images, name_to_index) = parse_text_images(&lines);

    struct FrameLine {
        line_index: usize,
        prefix: String,
        image_index: usize,
        address: u64,
    }

    let mut frame_lines: Vec<FrameLine> = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        if let Some((prefix, name, address)) = parse_frame_line(line) {
            if let Some(&image_index) = name_to_index.get(name.as_str()) {
                frame_lines.push(FrameLine {
                    line_index,
                    prefix,
                    image_index,
                    address,
                });
            }
        }
    }

    let requests: Vec<(usize, usize, u64)> = frame_lines
        .iter()
        .map(|frame| (frame.line_index, frame.image_index, frame.address))
        .collect();
    let results = symbolize_grouped(&images, &requests, options);

    let mut rewritten: HashMap<usize, String> = HashMap::new();
    for frame in &frame_lines {
        if let Some(SymbolizeOutcome::Resolved(resolved)) = results.get(&frame.line_index) {
            rewritten.insert(
                frame.line_index,
                format!("{} {}", frame.prefix, format_crash_symbol(resolved)),
            );
        }
    }

    let mut output = String::with_capacity(input.len());
    for (line_index, line) in lines.iter().enumerate() {
        match rewritten.get(&line_index) {
            Some(replacement) => output.push_str(replacement),
            None => output.push_str(line),
        }
        output.push('\n');
    }
    output
}

// Parses the "Binary Images:" table, returning the images and a name->index map.
fn parse_text_images<'a>(lines: &[&'a str]) -> (Vec<Image>, HashMap<&'a str, usize>) {
    let mut images = Vec::new();
    let mut name_to_index = HashMap::new();

    let Some(start) = lines
        .iter()
        .position(|line| line.trim_start().starts_with("Binary Images:"))
    else {
        return (images, name_to_index);
    };

    for line in &lines[start + 1..] {
        if line.trim().is_empty() {
            break;
        }
        if let Some((name, image)) = parse_text_image_line(line) {
            name_to_index.entry(name).or_insert(images.len());
            images.push(image);
        }
    }

    (images, name_to_index)
}

fn parse_text_image_line(line: &str) -> Option<(&str, Image)> {
    // 0xSTART - 0xEND  name arch  <uuid>  /path
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 6 || tokens[1] != "-" {
        return None;
    }
    let base = parse_hex(tokens[0])?;
    let name = tokens[3].trim_start_matches('+');
    let uuid_token = tokens[5];
    let uuid_hex = uuid_token
        .starts_with('<')
        .then(|| normalize_hex_id(uuid_token));
    Some((name, Image { uuid_hex, base }))
}

fn parse_frame_line(line: &str) -> Option<(String, String, u64)> {
    // FRAME_NO  image_name  0xADDRESS  <unsymbolicated detail>
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 4 || !tokens[0].bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let address_token = tokens[2];
    if !address_token.starts_with("0x") && !address_token.starts_with("0X") {
        return None;
    }
    let address = parse_hex(address_token)?;
    let address_end = line.find(address_token)? + address_token.len();
    Some((
        line[..address_end].to_string(),
        tokens[1].to_string(),
        address,
    ))
}

fn format_crash_symbol(frame: &SymbolizedFrame) -> String {
    match &frame.location {
        Some(location) => format!("{} ({}:{})", frame.symbol, location.file, location.line),
        None => format!("{} + {}", frame.symbol, frame.offset),
    }
}

fn parse_hex(token: &str) -> Option<u64> {
    let value = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
        .unwrap_or(token);
    u64::from_str_radix(value, 16).ok()
}
