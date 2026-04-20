use hexi::args::Args;
use hexi::hex_read::{dump_hex, dump_hex_to};
use std::time::Instant;

fn args(disable_header: bool, color_no: bool, offsets_no: bool, no_hex: bool, ascii_no: bool) -> Args {
    Args {
        tui_no: false,
        disable_header,
        color_no,
        offsets_no,
        no_hex,
        ascii_no,
        force_large: false,
        file_name: None,
    }
}

fn render(data: &[u8], a: &Args) -> String {
    let mut out = Vec::new();
    dump_hex_to(data, a, &mut out);
    String::from_utf8(out).unwrap()
}

// ── output correctness ────────────────────────────────────────────────────────

#[test]
fn hex_encodes_bytes_correctly() {
    // disable_header=true, no color, no offset, hex only, no ascii
    let out = render(b"\x00\xff\x41", &args(true, true, true, false, true));
    assert!(out.contains("00 ff 41"), "unexpected output: {out:?}");
}

#[test]
fn ascii_shows_printable_chars_and_dots_for_non_printable() {
    // ascii only, no header, no offset, no hex
    let out = render(b"A\x00B", &args(true, true, true, true, false));
    assert!(out.contains("A.B"), "unexpected output: {out:?}");
}

#[test]
fn offsets_increment_by_16_per_row() {
    let data = vec![b'A'; 32]; // exactly 2 rows
    let out = render(&data, &args(true, true, false, true, true));
    assert!(out.contains("00000000"), "missing first offset: {out:?}");
    assert!(out.contains("00000010"), "missing second offset: {out:?}");
}

#[test]
fn partial_last_row_pads_hex_to_full_width() {
    // 1 byte: hex "ab" + 45 padding spaces + "\t\t\n"
    let out = render(b"\xab", &args(true, true, true, false, true));
    // 15 missing bytes × 3 chars = 45 spaces
    let expected = format!("ab{}\t\t\n", " ".repeat(45));
    assert_eq!(out, expected, "wrong padding: {out:?}");
}

#[test]
fn exactly_16_bytes_produces_one_row_no_padding() {
    let data = b"0123456789abcdef";
    let out = render(data, &args(true, true, true, false, true));
    // full row: no trailing spaces before the tab separator
    assert!(
        out.contains("30 31 32 33 34 35 36 37 38 39 61 62 63 64 65 66\t\t"),
        "unexpected output: {out:?}"
    );
    assert_eq!(out.lines().count(), 1);
}

#[test]
fn header_present_by_default() {
    let out = render(b"x", &args(false, true, false, false, false));
    assert!(out.contains("offset"), "header missing: {out:?}");
    assert!(out.contains("ascii"), "header missing ascii column: {out:?}");
}

#[test]
fn header_omitted_when_disabled() {
    let out = render(b"x", &args(true, true, false, false, false));
    assert!(!out.contains("offset"), "header should not appear: {out:?}");
}

#[test]
fn empty_input_renders_only_header_and_separator() {
    let out = render(b"", &args(false, true, false, false, false));
    assert_eq!(out.lines().count(), 2, "expected header + separator only: {out:?}");
}

#[test]
fn no_hex_flag_omits_hex_column() {
    let out = render(b"A", &args(true, true, false, true, false));
    assert!(!out.contains("41"), "hex should be absent: {out:?}");
    assert!(out.contains('A'), "ascii should still appear: {out:?}");
}

#[test]
fn ascii_no_flag_omits_ascii_column() {
    let out = render(b"A", &args(true, true, false, false, true));
    assert!(out.contains("41"), "hex should appear: {out:?}");
    // the literal char 'A' must not appear as an ASCII column entry
    let hex_line = out.lines().next().unwrap();
    assert!(!hex_line.ends_with('A'), "ascii column should be absent: {out:?}");
}

// ── color output ──────────────────────────────────────────────────────────────

#[test]
fn printable_bytes_colored_green() {
    // color enabled (color_no=false)
    let out = render(b"Hello", &args(true, false, true, false, true));
    assert!(out.contains("\x1b[32m"), "missing green escape: {out:?}");
    assert!(out.contains("\x1b[0m"), "missing reset: {out:?}");
}

#[test]
fn zero_bytes_colored_bright_black() {
    let out = render(b"\x00\x00", &args(true, false, true, false, true));
    assert!(out.contains("\x1b[90m"), "missing bright-black escape: {out:?}");
}

#[test]
fn ff_bytes_colored_bright_white() {
    let out = render(b"\xff\xff", &args(true, false, true, false, true));
    assert!(out.contains("\x1b[97m"), "missing bright-white escape: {out:?}");
}

#[test]
fn uncolored_output_has_no_ansi_escapes() {
    let out = render(b"Hello\x00\xff", &args(true, true, false, false, false));
    assert!(!out.contains("\x1b["), "unexpected escape in uncolored output: {out:?}");
}

// ── smoke / stability ─────────────────────────────────────────────────────────

#[test]
fn dump_hex_does_not_panic_on_varied_input() {
    let data: Vec<u8> = (0u8..=255).collect();
    dump_hex(&data, &args(false, true, false, false, false));
}

#[test]
fn hex_dump_performance() {
    let data = vec![0x42u8; 1024 * 1024]; // 1 MB
    let a = args(false, true, false, false, false);
    let start = Instant::now();
    dump_hex(&data, &a);
    assert!(start.elapsed().as_secs() < 5, "dump too slow: {:?}", start.elapsed());
}
