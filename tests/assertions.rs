use hexi::args::Args;
use std::time::Instant;

#[test]
fn fool_check_all_disabled() {
    // fool_check should be true only when ALL three are disabled
    let args = Args {
        tui_no: false,
        disable_header: false,
        color_no: false,
        offsets_no: true,
        no_hex: true,
        ascii_no: true,
        force_large: false,
        file_name: None,
    };
    assert!(args.fool_check());
}

#[test]
fn fool_check_any_enabled() {
    // if any output is enabled, fool_check should be false
    let args = Args {
        tui_no: false,
        disable_header: false,
        color_no: false,
        offsets_no: false,
        no_hex: true,
        ascii_no: true,
        force_large: false,
        file_name: None,
    };
    assert!(!args.fool_check());
}

#[test]
fn fool_check_performance() {
    // should be fast af
    let args = Args {
        tui_no: false,
        disable_header: false,
        color_no: false,
        offsets_no: true,
        no_hex: true,
        ascii_no: true,
        force_large: false,
        file_name: None,
    };

    let start = Instant::now();
    for _ in 0..100_000 {
        let _ = args.fool_check();
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 10,
        "fool_check too slow: {:?}ms",
        elapsed.as_millis()
    );
}

#[test]
fn args_creation_speed() {
    // creating Args shouldn't be slow
    let start = Instant::now();
    for _ in 0..10_000 {
        let _args = Args {
            tui_no: false,
            disable_header: false,
            color_no: false,
            offsets_no: false,
            no_hex: false,
            ascii_no: false,
            force_large: false,
            file_name: None,
        };
    }
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 50, "args creation too slow");
}
