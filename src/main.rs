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

mod args;
mod hex_read;

fn main() -> Result<(), u8> {
    let args = args::Args::parse();
    if args.fool_check() {
        eprintln!("\x1b[1;31merror: \x1b[0;1mwhy do you want to display nothing?\x1b[0m");
        return Err(2);
    }
    if !args.tui_no {
        eprintln!("\x1b[1;33mwarn: \x1b[0;1mTUI is not implemented yet. using dump mode (use -t)\x1b[0m");
    }
    let data = args.read_input().unwrap();
    hex_read::dump_hex(&*data, &args);
    Ok(())
}
