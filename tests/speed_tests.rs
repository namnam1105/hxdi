use hexi::args::Args;
use hexi::hex_read::dump_hex;
use std::time::Instant;

#[test]
fn file_read_speed() {
    // test reading a reasonable file size

    let start = Instant::now();
    // create some dummy data like we'd read from a file
    let _data = vec![0x42u8; 1024 * 1024]; // 1MB
    let elapsed = start.elapsed();

    println!("File creation: {:?}ms", elapsed.as_millis());
    assert!(elapsed.as_secs() < 1, "shouldn't take forever");
}

#[test]
fn dump_to_stdout_speed() {
    // measure how fast we can dump hex to stdout
    let data = vec![0xAB; 512 * 1024]; // 512KB
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

    let throughput = (data.len() as f64 / 1024.0) / elapsed.as_secs_f64();
    println!("Dump throughput: {:.2} KB/s", throughput);
    assert!(elapsed.as_secs() < 3, "dump too slow");
}

#[test]
fn colored_output_speed() {
    // check that colored output doesn't tank performance too bad
    let data = vec![0x00, 0xFF, 0x42]; // some varied bytes
    let data: Vec<u8> = data.iter().cycle().take(1024 * 256).copied().collect();

    let args_uncolored = Args {
        tui_no: false,
        disable_header: false,
        color_no: true,
        offsets_no: false,
        no_hex: false,
        ascii_no: false,
        force_large: false,
        file_name: None,
    };

    let args_colored = Args {
        tui_no: false,
        disable_header: false,
        color_no: false,
        offsets_no: false,
        no_hex: false,
        ascii_no: false,
        force_large: false,
        file_name: None,
    };

    let start1 = Instant::now();
    dump_hex(&data, &args_uncolored);
    let time1 = start1.elapsed();

    let start2 = Instant::now();
    dump_hex(&data, &args_colored);
    let time2 = start2.elapsed();

    println!(
        "Uncolored: {:?}ms, Colored: {:?}ms",
        time1.as_millis(),
        time2.as_millis()
    );
    // both should be reasonable
    assert!(time1.as_secs() < 2);
    assert!(time2.as_secs() < 2);
}

#[test]
fn very_large_dump() {
    // what happens with a big chunk
    let data = vec![0x67u8; 10 * 1024 * 1024]; // 10MB

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

    println!("10MB dump: {:?}s", elapsed.as_secs_f64());
    assert!(elapsed.as_secs() < 10, "10MB dump shouldn't take forever");
}
