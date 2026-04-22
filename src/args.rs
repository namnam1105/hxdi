/*
hxdi - a TUI hex editor
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
use std::io;
use std::io::Read;

/// a TUI hex editor/dumper
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// This disables the interactive TUI interface (read-only)
    #[arg(short, long)]
    pub tui_no: bool,
    /// This disables the table header
    #[arg(short, long)]
    pub disable_header: bool,
    /// This disables colored output
    #[arg(short, long)]
    pub color_no: bool,
    /// This disables the offset column
    #[arg(short, long)]
    pub offsets_no: bool,
    /// This disables the hex data output
    #[arg(short, long)]
    pub no_hex: bool,
    /// This disables the ascii data output
    #[arg(short, long)]
    pub ascii_no: bool,
    /// Ignore the large file warning
    #[arg(short, long)]
    pub force_large: bool,
    /// The file name to read from
    pub file_name: Option<String>, // option cus sometimes we want to push into stdin data.
}

impl Args {
    /// this reads input to bytes either from a pipe or from a file
    pub fn read_input(&self) -> io::Result<Vec<u8>> {
        match &self.file_name {
            Some(path) => {
                let size = std::fs::metadata(path)?.len();
                if size > 100 * 1024 * 1024 && !self.force_large {
                    eprintln!(
                        "\x1b[31;1merror: \x1b[0;1mfile is too large ({}MB), use --force-large to open anyway\x1b[0m",
                        size / 1024 / 1024
                    );
                    std::process::exit(1);
                }
                std::fs::read(path)
            }
            None => {
                let mut buf = Vec::new();
                io::stdin().read_to_end(&mut buf)?;
                Ok(buf)
            }
        }
    }

    /// this checks for everything being disabled
    /// the name is a joke, if you are offended then I apologize <3
    pub fn fool_check(&self) -> bool {
        self.ascii_no && self.no_hex && self.offsets_no // passed the fool check
    }
}
