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

const RESET: &[u8] = b"\x1b[0m";
const GREEN: &[u8] = b"\x1b[32m";
const BRIGHT_BLACK: &[u8] = b"\x1b[90m";
const BRIGHT_WHITE: &[u8] = b"\x1b[97m";

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
// 48 spaces covers the maximum padding of 15 missing bytes * 3 chars = 45
const SPACES: &[u8; 48] = b"                                                ";
const DOTS: &[u8; 16] = b"................";

#[inline(always)]
fn hex_byte(b: u8) -> [u8; 2] {
    [HEX_CHARS[(b >> 4) as usize], HEX_CHARS[(b & 0xf) as usize]]
}

// Builds "XX XX XX ..." into a stack buffer and writes it in one call.
fn write_hex(out: &mut impl Write, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    // Max 16 bytes: 2 hex chars + 15 * (1 space + 2 hex) = 47 bytes
    let mut buf = [0u8; 47];
    let hb = hex_byte(bytes[0]);
    buf[0] = hb[0];
    buf[1] = hb[1];
    let mut pos = 2;
    for &b in &bytes[1..] {
        buf[pos] = b' ';
        let hb = hex_byte(b);
        buf[pos + 1] = hb[0];
        buf[pos + 2] = hb[1];
        pos += 3;
    }
    out.write_all(&buf[..pos]).unwrap();
}

// Writes colored hex, batching consecutive bytes that share the same color
// category to minimize the number of escape-sequence writes.
fn write_hex_colored(out: &mut impl Write, bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        if i > 0 {
            out.write_all(b" ").unwrap();
        }
        let b = bytes[i];
        match b {
            0x00 => {
                let start = i;
                while i < bytes.len() && bytes[i] == 0x00 {
                    i += 1;
                }
                out.write_all(BRIGHT_BLACK).unwrap();
                for j in start..i {
                    if j > start {
                        out.write_all(b" ").unwrap();
                    }
                    out.write_all(&hex_byte(0x00)).unwrap();
                }
                out.write_all(RESET).unwrap();
            }
            32..=126 => {
                let start = i;
                while i < bytes.len() && (32..=126).contains(&bytes[i]) {
                    i += 1;
                }
                out.write_all(GREEN).unwrap();
                for j in start..i {
                    if j > start {
                        out.write_all(b" ").unwrap();
                    }
                    out.write_all(&hex_byte(bytes[j])).unwrap();
                }
                out.write_all(RESET).unwrap();
            }
            0xFF => {
                let start = i;
                while i < bytes.len() && bytes[i] == 0xFF {
                    i += 1;
                }
                out.write_all(BRIGHT_WHITE).unwrap();
                for j in start..i {
                    if j > start {
                        out.write_all(b" ").unwrap();
                    }
                    out.write_all(&hex_byte(0xFF)).unwrap();
                }
                out.write_all(RESET).unwrap();
            }
            _ => {
                let start = i;
                while i < bytes.len() && !matches!(bytes[i], 0x00 | 32..=126 | 0xFF) {
                    i += 1;
                }
                for j in start..i {
                    if j > start {
                        out.write_all(b" ").unwrap();
                    }
                    out.write_all(&hex_byte(bytes[j])).unwrap();
                }
            }
        }
    }
}

// Converts the chunk to printable chars in a stack buffer, then writes once.
fn write_ascii(out: &mut impl Write, bytes: &[u8]) {
    let mut buf = [0u8; 16];
    for (i, &b) in bytes.iter().enumerate() {
        buf[i] = if (32..=126).contains(&b) { b } else { b'.' };
    }
    out.write_all(&buf[..bytes.len()]).unwrap();
}

// Writes colored ASCII, batching consecutive printable bytes as a raw slice
// (no per-char format overhead) and grouping dots under shared color escapes.
fn write_ascii_colored(out: &mut impl Write, bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            0x00 => {
                let start = i;
                while i < bytes.len() && bytes[i] == 0x00 {
                    i += 1;
                }
                out.write_all(BRIGHT_BLACK).unwrap();
                out.write_all(&DOTS[..i - start]).unwrap();
                out.write_all(RESET).unwrap();
            }
            32..=126 => {
                let start = i;
                while i < bytes.len() && (32..=126).contains(&bytes[i]) {
                    i += 1;
                }
                out.write_all(GREEN).unwrap();
                out.write_all(&bytes[start..i]).unwrap();
                out.write_all(RESET).unwrap();
            }
            _ => {
                let start = i;
                while i < bytes.len() && !matches!(bytes[i], 0x00 | 32..=126) {
                    i += 1;
                }
                out.write_all(&DOTS[..i - start]).unwrap();
            }
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

fn print_row(
    out: &mut impl Write,
    show_offset: bool,
    show_hex: bool,
    show_ascii: bool,
    colored: bool,
    offset: usize,
    data: &[u8],
) {
    if show_offset {
        // Write "XXXXXXXX\t" without format!() overhead
        let mut buf = [0u8; 9];
        let mut o = offset;
        for i in (0..8).rev() {
            buf[i] = HEX_CHARS[o & 0xf];
            o >>= 4;
        }
        buf[8] = b'\t';
        out.write_all(&buf).unwrap();
    }

    if show_hex {
        if colored {
            write_hex_colored(out, data);
        } else {
            write_hex(out, data);
        }
        let padding = (16 - data.len()) * 3;
        out.write_all(&SPACES[..padding]).unwrap();
        out.write_all(b"\t\t").unwrap();
    }

    if show_ascii {
        if colored {
            write_ascii_colored(out, data);
        } else {
            write_ascii(out, data);
        }
    }

    out.write_all(b"\n").unwrap();
}

pub fn dump_hex_to<W: Write>(bytes: &[u8], args: &Args, out: &mut W) {
    if !args.disable_header {
        draw_table_header(out, args);
    }

    let show_offset = !args.offsets_no;
    let show_hex = !args.no_hex;
    let show_ascii = !args.ascii_no;
    let colored = !args.color_no;

    for (i, chunk) in bytes.chunks(16).enumerate() {
        print_row(out, show_offset, show_hex, show_ascii, colored, i * 16, chunk);
    }
}

pub fn dump_hex(bytes: &[u8], args: &Args) {
    let stdout = stdout();
    // 64 KB buffer reduces system call frequency vs the default 8 KB
    let mut out = BufWriter::with_capacity(1 << 16, stdout.lock());
    dump_hex_to(bytes, args, &mut out);
}
