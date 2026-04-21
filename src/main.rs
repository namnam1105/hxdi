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

use clap::Parser;
use hexi::args;
use hexi::hex_read;
use hexi::tui;

fn main() -> std::io::Result<()> {
    let args = args::Args::parse();
    if args.fool_check() {
        eprintln!("\x1b[1;31merror: \x1b[0;1mwhy do you want to display nothing?\x1b[0m");
        std::process::exit(2);
    }
    let data = args.read_input().unwrap();
    if !args.tui_no {
        tui::run(
            data,
            args.file_name,
            !args.color_no,
            !args.disable_header,
            !args.offsets_no,
            !args.no_hex,
            !args.ascii_no,
        )?;
    } else {
        hex_read::dump_hex(&*data, &args);
    }
    Ok(())
}
