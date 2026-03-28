use hexi::args::Args;
use hexi::hex_read::dump_hex;
use std::time::Instant;

#[test]
fn hex_dump_works() {
    // just make sure dump_hex doesn't crash
    let data = b"Hello, World!";
    let args = Args {
        tui_no: false,
        disable_header: false,
        color_no: true,
        offsets_no: false,
        no_hex: false,
        ascii_no: false,
        force_large: false,
        file_name: None,
    };

    // it writes to stdout so we can't really capture it easily
    // but at least make sure it doesn't panic
    dump_hex(data, &args);
}

#[test]
fn hex_dump_with_all_flags() {
    // try with different flag combinations
    let data = b"test";
    let args = Args {
        tui_no: false,
        disable_header: true,
        color_no: true,
        offsets_no: true,
        no_hex: false,
        ascii_no: true,
        force_large: false,
        file_name: None,
    };

    dump_hex(data, &args);
}

#[test]
fn hex_dump_performance() {
    // make sure dumping doesn't suck
    let data = vec![0x42u8; 1024 * 1024]; // 1MB
    let args = Args {
        tui_no: false,
        disable_header: false,
        color_no: true,
        offsets_no: false,
        no_hex: false,
        ascii_no: false,
        force_large: false,
        file_name: None,
    };

    let start = Instant::now();
    dump_hex(&data, &args);
    let elapsed = start.elapsed();

    // should be reasonably fast
    assert!(
        elapsed.as_secs() < 5,
        "dump_hex too slow: {:?}s",
        elapsed.as_secs_f64()
    );
}

#[test]
fn hex_dump_large_data() {
    // make sure it handles larger chunks ok
    let data = vec![0xFF; 16 * 100]; // 100 rows
    let args = Args {
        tui_no: false,
        disable_header: false,
        color_no: true,
        offsets_no: false,
        no_hex: false,
        ascii_no: false,
        force_large: false,
        file_name: None,
    };

    dump_hex(&data, &args);
}
