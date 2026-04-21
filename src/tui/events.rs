use crate::tui::{actions::Action, app::App, types::*};
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};

pub fn handle_event(app: &mut App, event: Event) -> Action {
    match event {
        Event::Mouse(m) => handle_mouse(app, m),
        _ => match &app.dialog {
            Dialog::None => handle_normal(app, event),
            _ => Action::None,
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
                app.drag_origin = Some(idx); // remember click origin for drag-select
                ensure_visible(app);
            } else {
                app.drag_origin = None;
            }
            Action::None
        }
        MouseEventKind::Drag(MouseButton::Left) if app.dragging => {
            // Resize bpr by dragging separator
            let hex_inner = event.column
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

    if app.show_hex
        && col >= app.hex_content_x
        && col < app.hex_content_x + app.hex_content_w
    {
        let byte_col = ((col - app.hex_content_x) / 3) as usize;
        if byte_col < app.bytes_per_row {
            let idx = data_row * app.bytes_per_row + byte_col;
            if idx < app.data.len() { return Some(idx); }
        }
    }

    if app.show_ascii
        && col >= app.ascii_content_x
        && col < app.ascii_content_x + app.ascii_content_w
    {
        let byte_col = (col - app.ascii_content_x) as usize;
        if byte_col < app.bytes_per_row {
            let idx = data_row * app.bytes_per_row + byte_col;
            if idx < app.data.len() { return Some(idx); }
        }
    }

    None
}

fn handle_normal(app: &mut App, event: Event) -> Action {
    let Event::Key(key) = event else { return Action::None; };

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
            set_status(app, "Saved.");
            Action::SaveFile
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
            if app.sel_anchor.is_none() { app.sel_anchor = Some(app.cursor); }
            move_cursor(app, -1);
            Action::None
        }
        (KeyModifiers::SHIFT, KeyCode::Right) => {
            if app.sel_anchor.is_none() { app.sel_anchor = Some(app.cursor); }
            move_cursor(app, 1);
            Action::None
        }
        (KeyModifiers::SHIFT, KeyCode::Up) => {
            if app.sel_anchor.is_none() { app.sel_anchor = Some(app.cursor); }
            move_cursor(app, -(app.bytes_per_row as i64));
            Action::None
        }
        (KeyModifiers::SHIFT, KeyCode::Down) => {
            if app.sel_anchor.is_none() { app.sel_anchor = Some(app.cursor); }
            move_cursor(app, app.bytes_per_row as i64);
            Action::None
        }

        // pane switch
        (KeyModifiers::NONE, KeyCode::Tab) => {
            app.active_pane = match app.active_pane {
                ActivePane::Hex   => ActivePane::Ascii,
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
                if (32..=126).contains(&b) { (b as char).to_string() }
                else { format!("\\x{:02x}", b) }
            })
            .collect::<String>(),
    };

    copy_osc52(&text);
    let label = if bytes.len() == 1 { "1 byte".to_string() } else { format!("{} bytes", bytes.len()) };
    set_status(app, &format!("Copied {label}"));
}

fn set_status(app: &mut App, msg: &str) {
    app.status_msg = Some(msg.to_string());
    app.status_msg_until = Some(
        std::time::Instant::now() + std::time::Duration::from_secs(3),
    );
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
        out.push(if chunk.len() > 1 { CHARS[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[(n & 63) as usize] as char } else { '=' });
    }
    out
}
