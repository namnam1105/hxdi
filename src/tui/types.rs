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

#[derive(Clone, Copy, PartialEq)]
pub enum EditMode {
    Overwrite,
    Insert,
    ReadOnly,
}
#[derive(Clone, Copy, PartialEq)]
pub enum ActivePane {
    Hex,
    Ascii,
}
#[derive(Clone, Copy, PartialEq)]
pub enum NibbleHalf {
    High,
    Low,
}

#[derive(Clone, PartialEq)]
pub enum Dialog {
    None,
    UnsavedChanges(UnsavedFocus),
    Find(FindState),
    Goto(GotoState),
}
impl Default for Dialog {
    fn default() -> Self {
        Dialog::None
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum UnsavedFocus {
    Save,
    DontSave,
    Cancel,
}
#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode {
    Hex,
    Ascii,
}
#[derive(Clone, Copy, PartialEq)]
pub enum GotoMode {
    Offset,
    Value,
    Ascii,
}

#[derive(Clone, PartialEq, Default)]
pub struct FindState {
    pub input: String,
    pub mode: SearchMode,
    pub last_match: Option<usize>,
}
#[derive(Clone, PartialEq, Default)]
pub struct GotoState {
    pub input: String,
    pub mode: GotoMode,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::Ascii
    }
}
impl Default for GotoMode {
    fn default() -> Self {
        GotoMode::Offset
    }
}
