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

use std::io::ErrorKind;

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
    let data = args.read_input();
    let data = match data {
        Ok(data) => data,
        Err(e) => {
            match e.kind() {
                ErrorKind::PermissionDenied => {
                    eprintln!("\x1b[31;1merror: \x1b[0;1mnot enough permissions to open the file.")
                }
                _ => eprintln!("\x1b[31;1merror: \x1b[0;1mfailed to open the file."),
            }
            std::process::exit(1)
        }
    };
    if !args.tui_no {
        let read_only = args
            .file_name
            .as_deref()
            .map(|p| std::fs::OpenOptions::new().write(true).open(p).is_err())
            .unwrap_or(false);
        tui::run(
            data,
            args.file_name,
            read_only,
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
