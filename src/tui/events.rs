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

use crate::tui::{actions::Action, app::App, types::*};
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};

pub fn handle_event(app: &mut App, event: Event) -> Action {
    match event {
        Event::Mouse(m) => handle_mouse(app, m),
        _ => match app.dialog.clone() {
            Dialog::None => handle_normal(app, event),
            Dialog::UnsavedChanges(focus) => handle_unsaved(app, focus, event),
            Dialog::Find(state) => handle_find(app, state, event),
            Dialog::Goto(state) => handle_goto(app, state, event),
        },
    }
}

fn handle_mouse(app: &mut App, event: crossterm::event::MouseEvent) -> Action {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Separator drag takes priority
            if event.column >= app.sep_col && event.column <= app.sep_col + 1 {
                app.dragging = true;
                app.drag_origin = None;
            } else if let Some(idx) = mouse_to_byte(app, event.column, event.row) {
                app.sel_anchor = None;
                app.cursor = idx;
                app.drag_origin = Some(idx);
                app.nibble = NibbleHalf::High;
                if app.show_hex
                    && event.column >= app.hex_content_x
                    && event.column < app.hex_content_x + app.hex_content_w
                {
                    app.active_pane = ActivePane::Hex;
                } else if app.show_ascii
                    && event.column >= app.ascii_content_x
                    && event.column < app.ascii_content_x + app.ascii_content_w
                {
                    app.active_pane = ActivePane::Ascii;
                }
                ensure_visible(app);
            } else {
                app.drag_origin = None;
            }
            Action::None
        }
        MouseEventKind::Drag(MouseButton::Left) if app.dragging => {
            // Resize bpr by dragging separator
            let hex_inner = event
                .column
                .saturating_sub(app.offset_w + 1)
                .saturating_sub(1);
            app.bpr_override = Some(((hex_inner + 1) / 3).max(1) as usize);
            Action::None
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            // Plain drag = extend selection from click origin
            if let Some(origin) = app.drag_origin {
                if let Some(idx) = mouse_to_byte(app, event.column, event.row) {
                    if idx != origin {
                        app.sel_anchor = Some(origin);
                        app.cursor = idx;
                        ensure_visible(app);
                    }
                }
            }
            Action::None
        }
        MouseEventKind::Up(MouseButton::Left) => {
            app.dragging = false;
            Action::None
        }
        MouseEventKind::ScrollUp => {
            app.scroll_row = app.scroll_row.saturating_sub(3);
            Action::None
        }
        MouseEventKind::ScrollDown => {
            let max = app.total_rows().saturating_sub(app.visible_rows);
            app.scroll_row = (app.scroll_row + 3).min(max);
            Action::None
        }
        _ => Action::None,
    }
}

fn mouse_to_byte(app: &App, col: u16, row: u16) -> Option<usize> {
    let content_row = row.checked_sub(app.editor_content_y)? as usize;
    let data_row = app.scroll_row + content_row;

    if app.show_hex && col >= app.hex_content_x && col < app.hex_content_x + app.hex_content_w {
        let byte_col = ((col - app.hex_content_x) / 3) as usize;
        if byte_col < app.bytes_per_row {
            let idx = data_row * app.bytes_per_row + byte_col;
            if idx < app.data.len() {
                return Some(idx);
            }
        }
    }

    if app.show_ascii
        && col >= app.ascii_content_x
        && col < app.ascii_content_x + app.ascii_content_w
    {
        let byte_col = (col - app.ascii_content_x) as usize;
        if byte_col < app.bytes_per_row {
            let idx = data_row * app.bytes_per_row + byte_col;
            if idx < app.data.len() {
                return Some(idx);
            }
        }
    }

    None
}

fn handle_unsaved(app: &mut App, focus: UnsavedFocus, event: Event) -> Action {
    let Event::Key(key) = event else {
        return Action::None;
    };
    match key.code {
        KeyCode::Left | KeyCode::Right => {
            app.dialog = Dialog::UnsavedChanges(match focus {
                UnsavedFocus::Save => {
                    if key.code == KeyCode::Right {
                        UnsavedFocus::DontSave
                    } else {
                        UnsavedFocus::Cancel
                    }
                }
                UnsavedFocus::DontSave => {
                    if key.code == KeyCode::Right {
                        UnsavedFocus::Cancel
                    } else {
                        UnsavedFocus::Save
                    }
                }
                UnsavedFocus::Cancel => {
                    if key.code == KeyCode::Right {
                        UnsavedFocus::Save
                    } else {
                        UnsavedFocus::DontSave
                    }
                }
            });
            Action::None
        }
        KeyCode::Enter => {
            app.dialog = Dialog::None;
            match focus {
                UnsavedFocus::Save => Action::SaveQuit,
                UnsavedFocus::DontSave => Action::Quit,
                UnsavedFocus::Cancel => Action::None,
            }
        }
        KeyCode::Esc => {
            app.dialog = Dialog::None;
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_find(app: &mut App, mut state: FindState, event: Event) -> Action {
    let Event::Key(key) = event else {
        return Action::None;
    };
    match key.code {
        KeyCode::Esc => {
            app.dialog = Dialog::None;
            Action::None
        }
        KeyCode::Tab => {
            state.mode = match state.mode {
                SearchMode::Ascii => SearchMode::Hex,
                SearchMode::Hex => SearchMode::Ascii,
            };
            app.dialog = Dialog::Find(state);
            Action::None
        }
        KeyCode::Enter => {
            let start = state.last_match.map(|m| m + 1).unwrap_or(app.cursor + 1);
            let found = find_in_data(app, &state, start).or_else(|| find_in_data(app, &state, 0));
            if let Some(pos) = found {
                state.last_match = Some(pos);
                app.cursor = pos;
                app.sel_anchor = None;
                ensure_visible(app);
            } else if !state.input.is_empty() {
                set_status(app, "Not found");
            }
            app.dialog = Dialog::Find(state);
            Action::None
        }
        KeyCode::Backspace => {
            state.input.pop();
            state.last_match = None;
            app.dialog = Dialog::Find(state);
            Action::None
        }
        KeyCode::Char(c) => {
            state.input.push(c);
            state.last_match = None;
            app.dialog = Dialog::Find(state);
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_goto(app: &mut App, mut state: GotoState, event: Event) -> Action {
    let Event::Key(key) = event else {
        return Action::None;
    };
    match key.code {
        KeyCode::Esc => {
            app.dialog = Dialog::None;
            Action::None
        }
        KeyCode::Tab => {
            state.mode = match state.mode {
                GotoMode::Offset => GotoMode::Value,
                GotoMode::Value => GotoMode::Ascii,
                GotoMode::Ascii => GotoMode::Offset,
            };
            state.input.clear();
            app.dialog = Dialog::Goto(state);
            Action::None
        }
        KeyCode::Enter => {
            let target: Option<usize> = match state.mode {
                GotoMode::Offset => {
                    usize::from_str_radix(state.input.trim_start_matches("0x"), 16).ok()
                }
                GotoMode::Value => u8::from_str_radix(&state.input, 16)
                    .ok()
                    .and_then(|v| app.data.iter().position(|&b| b == v)),
                GotoMode::Ascii => state
                    .input
                    .chars()
                    .next()
                    .and_then(|c| app.data.iter().position(|&b| b == c as u8)),
            };
            app.dialog = Dialog::None;
            if let Some(pos) = target.filter(|&p| p < app.data.len()) {
                app.cursor = pos;
                app.sel_anchor = None;
                ensure_visible(app);
            } else {
                set_status(app, "Invalid / out of range");
            }
            Action::None
        }
        KeyCode::Backspace => {
            state.input.pop();
            app.dialog = Dialog::Goto(state);
            Action::None
        }
        KeyCode::Char(c) => {
            state.input.push(c);
            app.dialog = Dialog::Goto(state);
            Action::None
        }
        _ => Action::None,
    }
}

fn find_in_data(app: &App, state: &FindState, from: usize) -> Option<usize> {
    let pattern: Vec<u8> = match state.mode {
        SearchMode::Ascii => state.input.bytes().collect(),
        SearchMode::Hex => {
            let s: String = state
                .input
                .chars()
                .filter(|c| !c.is_ascii_whitespace())
                .collect();
            if s.is_empty() || s.len() % 2 != 0 {
                return None;
            }
            s.as_bytes()
                .chunks(2)
                .map(|c| u8::from_str_radix(std::str::from_utf8(c).ok()?, 16).ok())
                .collect::<Option<Vec<u8>>>()?
        }
    };
    if pattern.is_empty() {
        return None;
    }
    app.data[from..]
        .windows(pattern.len())
        .position(|w| w == pattern.as_slice())
        .map(|i| from + i)
}

fn handle_normal(app: &mut App, event: Event) -> Action {
    let Event::Key(key) = event else {
        return Action::None;
    };

    match (key.modifiers, key.code) {
        // quit / save
        (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
            if app.is_dirty() {
                app.dialog = Dialog::UnsavedChanges(UnsavedFocus::Save);
                Action::None
            } else {
                Action::Quit
            }
        }
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
            if app.edit_mode == EditMode::ReadOnly {
                set_status(app, "Read-only");
                return Action::None;
            }
            set_status(app, "Saved.");
            app.original = app.data.clone();
            Action::SaveFile
        }

        // open dialogs / toggle mode
        (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
            app.dialog = Dialog::Find(FindState::default());
            Action::None
        }
        (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
            app.dialog = Dialog::Goto(GotoState::default());
            Action::None
        }
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
            if app.edit_mode != EditMode::ReadOnly {
                app.edit_mode = match app.edit_mode {
                    EditMode::Overwrite => EditMode::Insert,
                    _ => EditMode::Overwrite,
                };
            }
            Action::None
        }

        // copy (pane-aware, selection-aware)
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            copy_selection(app);
            Action::None
        }

        // movement (clears selection)
        (KeyModifiers::NONE, KeyCode::Left) => {
            app.sel_anchor = None;
            move_cursor(app, -1);
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::Right) => {
            app.sel_anchor = None;
            move_cursor(app, 1);
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::Up) => {
            app.sel_anchor = None;
            move_cursor(app, -(app.bytes_per_row as i64));
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::Down) => {
            app.sel_anchor = None;
            move_cursor(app, app.bytes_per_row as i64);
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::PageUp) => {
            app.sel_anchor = None;
            let n = app.visible_rows as i64 * app.bytes_per_row as i64;
            move_cursor(app, -n);
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::PageDown) => {
            app.sel_anchor = None;
            let n = app.visible_rows as i64 * app.bytes_per_row as i64;
            move_cursor(app, n);
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::Home) => {
            app.sel_anchor = None;
            move_cursor(app, -(app.cursor_col() as i64));
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::End) => {
            app.sel_anchor = None;
            let remaining = (app.bytes_per_row - 1 - app.cursor_col()) as i64;
            move_cursor(app, remaining);
            Action::None
        }

        // selection movement (SHIFT+arrows)
        (KeyModifiers::SHIFT, KeyCode::Left) => {
            if app.sel_anchor.is_none() {
                app.sel_anchor = Some(app.cursor);
            }
            move_cursor(app, -1);
            Action::None
        }
        (KeyModifiers::SHIFT, KeyCode::Right) => {
            if app.sel_anchor.is_none() {
                app.sel_anchor = Some(app.cursor);
            }
            move_cursor(app, 1);
            Action::None
        }
        (KeyModifiers::SHIFT, KeyCode::Up) => {
            if app.sel_anchor.is_none() {
                app.sel_anchor = Some(app.cursor);
            }
            move_cursor(app, -(app.bytes_per_row as i64));
            Action::None
        }
        (KeyModifiers::SHIFT, KeyCode::Down) => {
            if app.sel_anchor.is_none() {
                app.sel_anchor = Some(app.cursor);
            }
            move_cursor(app, app.bytes_per_row as i64);
            Action::None
        }

        // pane switch
        (KeyModifiers::NONE, KeyCode::Tab) => {
            app.active_pane = match app.active_pane {
                ActivePane::Hex => ActivePane::Ascii,
                ActivePane::Ascii => ActivePane::Hex,
            };
            app.nibble = NibbleHalf::High;
            Action::None
        }

        // bpr resize
        (KeyModifiers::NONE, KeyCode::Char(',')) => {
            app.bpr_override = Some(app.bytes_per_row.saturating_sub(1).max(1));
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::Char('.')) => {
            app.bpr_override = Some(app.bytes_per_row + 1);
            Action::None
        }

        // offset column resize
        (KeyModifiers::NONE, KeyCode::Char('[')) => {
            app.offset_extra -= 1;
            Action::None
        }
        (KeyModifiers::NONE, KeyCode::Char(']')) => {
            app.offset_extra += 1;
            Action::None
        }

        // hex pane: overwrite nibble by nibble
        (KeyModifiers::NONE, KeyCode::Char(c))
            if app.active_pane == ActivePane::Hex && c.is_ascii_hexdigit() =>
        {
            if app.edit_mode == EditMode::ReadOnly {
                set_status(app, "Read-only");
                return Action::None;
            }
            let d = c.to_digit(16).unwrap() as u8;
            match app.nibble {
                NibbleHalf::High => {
                    app.pending_nibble = d;
                    app.nibble = NibbleHalf::Low;
                }
                NibbleHalf::Low => {
                    let byte = (app.pending_nibble << 4) | d;
                    if app.edit_mode == EditMode::Insert {
                        app.data.insert(app.cursor, byte);
                    } else {
                        app.data[app.cursor] = byte;
                    }
                    app.nibble = NibbleHalf::High;
                    move_cursor(app, 1);
                }
            }
            Action::None
        }

        // ascii pane: overwrite byte
        (KeyModifiers::NONE, KeyCode::Char(c)) if app.active_pane == ActivePane::Ascii => {
            if app.edit_mode == EditMode::ReadOnly {
                set_status(app, "Read-only");
                return Action::None;
            }
            if app.edit_mode == EditMode::Insert {
                app.data.insert(app.cursor, c as u8);
                move_cursor(app, 1);
            } else if !app.data.is_empty() {
                app.data[app.cursor] = c as u8;
                move_cursor(app, 1);
            }
            Action::None
        }

        _ => Action::None,
    }
}

fn move_cursor(app: &mut App, delta: i64) {
    let new = app.cursor as i64 + delta;
    let max = app.data.len().saturating_sub(1) as i64;
    app.cursor = new.clamp(0, max) as usize;
    ensure_visible(app);
}

fn ensure_visible(app: &mut App) {
    let row = app.cursor_row();
    if row < app.scroll_row {
        app.scroll_row = row;
    } else if app.visible_rows > 0 && row >= app.scroll_row + app.visible_rows {
        app.scroll_row = row.saturating_sub(app.visible_rows - 1);
    }
}

fn copy_selection(app: &mut App) {
    let bytes: Vec<u8> = if let Some((lo, hi)) = app.sel_range() {
        app.data[lo..=hi.min(app.data.len().saturating_sub(1))].to_vec()
    } else if let Some(&b) = app.data.get(app.cursor) {
        vec![b]
    } else {
        return;
    };

    let text = match app.active_pane {
        ActivePane::Hex => bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" "),
        ActivePane::Ascii => bytes
            .iter()
            .map(|&b| {
                if (32..=126).contains(&b) {
                    (b as char).to_string()
                } else {
                    format!("\\x{:02x}", b)
                }
            })
            .collect::<String>(),
    };

    copy_osc52(&text);
    let label = if bytes.len() == 1 {
        "1 byte".to_string()
    } else {
        format!("{} bytes", bytes.len())
    };
    set_status(app, &format!("Copied {label}"));
}

fn set_status(app: &mut App, msg: &str) {
    app.status_msg = Some(msg.to_string());
    app.status_msg_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
}

fn copy_osc52(text: &str) {
    use std::io::Write;
    let b64 = base64_encode(text.as_bytes());
    let _ = write!(std::io::stdout(), "\x1b]52;c;{b64}\x1b\\");
    let _ = std::io::stdout().flush();
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            CHARS[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            CHARS[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}
