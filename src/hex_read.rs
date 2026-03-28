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

use crate::args::Args;
use std::io::{BufWriter, Write, stdout};

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const BRIGHT_BLACK: &str = "\x1b[90m";
const BRIGHT_WHITE: &str = "\x1b[97m";

fn write_hex(out: &mut impl Write, bytes: &[u8]) {
    let mut first = true;
    for &b in bytes {
        if !first {
            out.write_all(b" ").unwrap();
        }
        write!(out, "{:02x}", b).unwrap();
        first = false;
    }
}

fn write_hex_colored(out: &mut impl Write, bytes: &[u8]) {
    let mut first = true;
    for &b in bytes {
        if !first {
            out.write_all(b" ").unwrap();
        }
        match b {
            0x00 => write!(out, "{}{:02x}{}", BRIGHT_BLACK, b, RESET).unwrap(),
            32..=126 => write!(out, "{}{:02x}{}", GREEN, b, RESET).unwrap(),
            0xFF => write!(out, "{}{:02x}{}", BRIGHT_WHITE, b, RESET).unwrap(),
            _ => write!(out, "{:02x}", b).unwrap(),
        }
        first = false;
    }
}

fn write_ascii(out: &mut impl Write, bytes: &[u8]) {
    for &b in bytes {
        let c: u8 = if (32..=126).contains(&b) { b } else { b'.' };
        out.write_all(&[c]).unwrap();
    }
}

fn write_ascii_colored(out: &mut impl Write, bytes: &[u8]) {
    for &b in bytes {
        match b {
            0x00 => write!(out, "{}.{}", BRIGHT_BLACK, RESET).unwrap(),
            32..=126 => write!(out, "{}{}{}", GREEN, b as char, RESET).unwrap(),
            _ => out.write_all(b".").unwrap(),
        }
    }
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
    if !args.offsets_no {
        parts.push("offset");
    }
    if !args.no_hex {
        parts.push("00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f");
    }
    if !args.ascii_no {
        parts.push("ascii\t\t");
    }

    let header = parts.join("\t\t");
    let separator = "─".repeat(visual_len(&header));

    writeln!(out, "{}", header).unwrap();
    writeln!(out, "{}", separator).unwrap();
}

fn print_row(out: &mut impl Write, args: &Args, offset: usize, data: &[u8]) {
    if !args.offsets_no {
        write!(out, "{:08x}\t", offset).unwrap();
    }

    if !args.no_hex {
        if args.color_no {
            write_hex(out, data);
        } else {
            write_hex_colored(out, data);
        }

        let padding = (16 - data.len()) * 3;
        for _ in 0..padding {
            out.write_all(b" ").unwrap();
        }
        out.write_all(b"\t\t").unwrap();
    }

    if !args.ascii_no {
        if args.color_no {
            write_ascii(out, data);
        } else {
            write_ascii_colored(out, data);
        }
    }

    out.write_all(b"\n").unwrap();
}

pub fn dump_hex(bytes: &[u8], args: &Args) {
    let stdout = stdout();
    let mut out = BufWriter::new(stdout.lock());
    if !args.disable_header {
        draw_table_header(&mut out, args);
    }
    for (i, chunk) in bytes.chunks(16).enumerate() {
        print_row(&mut out, args, i * 16, chunk);
    }
}
