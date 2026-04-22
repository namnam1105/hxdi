# hexi

hexi is a flexible hex-editor, allows flexible dumping options,
written in Rust, having near C dumping performance.

## how 2 use and compile

### compile

1. clone this repository
2. you need rust and cargo
3. cargo build --release

### use

Usage: hexi [OPTIONS] [FILE_NAME]

Arguments:
[FILE_NAME]  The file name to read from

Options:
* -t, --tui-no          This disables the interactive TUI interface (read-only)
* -d, --disable-header  This disables the table header
* -c, --color-no        This disables colored output
* -o, --offsets-no      This disables the offset column
* -n, --no-hex          This disables the hex data output
* -a, --ascii-no        This disables the ascii data output
* -f, --force-large     Ignore the large file warning
* -h, --help            Print help
* -V, --version         Print version

## plans

* [x] first working build
* [x] readme
* [x] license
* [x] unit tests
* [x] integrate `ratatui`
