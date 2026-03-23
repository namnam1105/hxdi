/*
hexi - a TUI hex editor
Copyright (C) 2026 namnam1105

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::io::{stdout, BufWriter, Write};
use crate::args::Args;

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const BRIGHT_BLACK: &str = "\x1b[90m";
const BRIGHT_WHITE: &str = "\x1b[97m";

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn to_hex_colored(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|&b| match b {
            0x00 => format!("{}{:02x}{}", BRIGHT_BLACK, b, RESET),
            32..=126 => format!("{}{:02x}{}", GREEN, b, RESET),
            0xFF => format!("{}{:02x}{}", BRIGHT_WHITE, b, RESET),
            _ => format!("{:02x}", b),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn to_ascii(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|&b| match b {
            32..=126 => (b as char).to_string(),
            _ => ".".to_string(),
        })
        .collect()
}

pub fn to_ascii_color(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|&b| match b {
            0x00 => format!("{}.{}", BRIGHT_BLACK, RESET),
            32..=126 => format!("{}{}{}", GREEN, b as char, RESET),
            _ => ".".to_string(),
        })
        .collect()
}

fn visual_len(s: &str) -> usize {
    let mut len = 0;
    for c in s.chars() {
        if c == '\t' {
            len = (len / 8 + 1) * 8;
        } else {
            len += 1;
        }
    }
    len
}

fn draw_table_header(out: &mut impl Write, args: &Args) {
    let mut parts: Vec<&str> = vec![];
    if !args.offsets_no { parts.push("offset"); }
    if !args.no_hex { parts.push("00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f"); }
    if !args.ascii_no { parts.push("ascii\t\t"); }

    let header = parts.join("\t\t");
    let separator = "─".repeat(visual_len(&header));

    writeln!(out, "{}", header).unwrap();
    writeln!(out, "{}", separator).unwrap();
}

fn offset_width(buf_size: usize) -> usize {
    if buf_size == 0 { return 8; }
    let hex_digits = (usize::BITS - buf_size.leading_zeros()) as usize;
    let tetrads = (hex_digits + 3) / 4;
    tetrads.max(8)
}

fn print_row(out: &mut impl Write, args: &Args, offset: usize, width: usize, data: &[u8]) {
    let offset_str = if args.offsets_no {
        "".to_string()
    } else {
        format!("{:0width$x}\t", offset)
    };
    let hex_str = if args.no_hex {
        "".to_string()
    } else {
        let hex = if args.color_no { to_hex(data) } else { to_hex_colored(data) };
        let padding = (16 - data.len()) * 3;
        format!("{}{}\t\t", hex, " ".repeat(padding))
    };
    let ascii_str = if args.ascii_no {
        "".to_string()
    } else if args.color_no {
        to_ascii(data)
    } else {
        to_ascii_color(data)
    };
    writeln!(out, "{}{}{}", offset_str, hex_str, ascii_str).unwrap();
}

pub fn dump_hex(bytes: &[u8], args: &Args) {
    let stdout = stdout();
    let mut out = BufWriter::new(stdout.lock());
    if !args.disable_header { draw_table_header(&mut out, args); }
    let width = offset_width(bytes.len());
    for (i, chunk) in bytes.chunks(16).enumerate() {
        print_row(&mut out, args, i * 16, width, chunk);
    }
}