use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::tui::app::App;
use crate::tui::types::*;

fn auto_offset_digits(file_size: usize) -> u16 {
    if file_size <= 0x10000 {
        4
    } else if file_size <= 0x1000000 {
        6
    } else {
        8
    }
}

fn auto_bpr(show_hex: bool, show_ascii: bool, available: u16) -> usize {
    match (show_hex, show_ascii) {
        (true, true) => ((available.saturating_sub(3)) / 4).max(1) as usize,
        (true, false) => ((available.saturating_sub(1) + 2) / 3).max(1) as usize,
        (false, true) => available.saturating_sub(2).max(1) as usize,
        (false, false) => 16,
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let frame_width = f.area().width;
    app.frame_width = frame_width;

    let base_digits = auto_offset_digits(app.data.len());
    let actual_digits = ((base_digits as i16 + app.offset_extra).max(2)) as u16;
    app.offset_digits = actual_digits;
    let offset_w: u16 = if app.show_offsets {
        actual_digits + 3
    } else {
        0
    };
    app.offset_w = offset_w;

    let sep_w: u16 = if app.show_hex && app.show_ascii { 1 } else { 0 };
    let inner_available = frame_width.saturating_sub(offset_w + sep_w);

    let max_bpr = auto_bpr(app.show_hex, app.show_ascii, inner_available);
    let bpr = app.bpr_override.unwrap_or(max_bpr).min(max_bpr).max(1);
    app.bytes_per_row = bpr;

    let hex_min_w: u16 = if app.show_hex { bpr as u16 * 3 + 1 } else { 0 };
    let ascii_w: u16 = if app.show_ascii { bpr as u16 + 2 } else { 0 };
    app.sep_col = offset_w + hex_min_w;

    // Vertical: title | separator | header | editor | hints
    let vert = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(if app.show_header { 1 } else { 0 }),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(f.area());

    let mk_horiz = |area: Rect| {
        Layout::horizontal([
            Constraint::Length(offset_w),
            Constraint::Min(hex_min_w),
            Constraint::Length(sep_w),
            Constraint::Length(ascii_w),
        ])
        .split(area)
    };

    let hdr_horiz = mk_horiz(vert[2]);
    let ed_horiz = mk_horiz(vert[3]);

    // Store bounds for mouse hit-testing
    app.editor_content_y = vert[3].y + 1;
    app.hex_content_x = ed_horiz[1].x + 1;
    app.hex_content_w = ed_horiz[1].width.saturating_sub(2);
    app.ascii_content_x = ed_horiz[3].x + 1;
    app.ascii_content_w = ed_horiz[3].width.saturating_sub(2);

    app.visible_rows = vert[3].height.saturating_sub(2) as usize;

    render_title(f, app, vert[0]);
    render_separator(f, vert[1]);
    if app.show_header {
        render_col_header(f, app, hdr_horiz[0], hdr_horiz[1], hdr_horiz[3]);
    }
    if app.show_offsets {
        render_offsets(f, app, ed_horiz[0]);
    }
    if app.show_hex {
        render_hex(f, app, ed_horiz[1]);
    }
    if app.show_ascii {
        render_ascii(f, app, ed_horiz[3]);
    }
    render_hints(f, app, vert[4]);
}

fn render_title(f: &mut Frame, app: &App, area: Rect) {
    let dirty = if app.is_dirty() { " [Modified]" } else { "" };
    let name = app.file_name.as_deref().unwrap_or("stdin");
    let content = match &app.status_msg {
        Some(msg) => format!(" hexi — {name}{dirty}  │  {msg}"),
        None => format!(" hexi — {name}{dirty}"),
    };
    f.render_widget(Paragraph::new(content).style(Style::new().reversed()), area);
}

fn render_separator(f: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    f.render_widget(
        Paragraph::new(line).style(Style::new().fg(Color::DarkGray)),
        area,
    );
}

fn render_col_header(
    f: &mut Frame,
    app: &App,
    offset_area: Rect,
    hex_area: Rect,
    ascii_area: Rect,
) {
    let dim = Style::new()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    if app.show_offsets {
        let w = app.offset_digits as usize;
        f.render_widget(
            Paragraph::new(format!(" {:>w$}", "offset")).style(dim),
            offset_area,
        );
    }
    if app.show_hex {
        let hex_hdr = (0..app.bytes_per_row)
            .map(|i| format!("{:02x}", i))
            .collect::<Vec<_>>()
            .join(" ");
        f.render_widget(Paragraph::new(format!(" {hex_hdr}")).style(dim), hex_area);
    }
    if app.show_ascii {
        f.render_widget(Paragraph::new(" ascii").style(dim), ascii_area);
    }
}

fn render_offsets(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let w = app.offset_digits as usize;
    let lines: Vec<Line> = (0..app.visible_rows)
        .map(|i| {
            let offset = (app.scroll_row + i) * app.bytes_per_row;
            Line::from(format!("{:0>w$x}", offset)).style(Style::new().fg(Color::DarkGray))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), inner);
}

pub fn byte_color(b: u8) -> Color {
    match b {
        0x00 => Color::DarkGray,
        32..=126 => Color::Green,
        0xFF => Color::White,
        _ => Color::Reset,
    }
}

const HEX_CHARS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

fn byte_style(app: &App, idx: usize, pane: ActivePane) -> Style {
    let b = app.data[idx];
    let is_cursor = idx == app.cursor;
    let pane_active = app.active_pane == pane;

    if is_cursor {
        return if pane_active {
            Style::new().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::new().bg(Color::DarkGray)
        };
    }

    let in_sel = app
        .sel_range()
        .map(|(lo, hi)| idx >= lo && idx <= hi)
        .unwrap_or(false);
    if in_sel {
        return Style::new().bg(Color::Blue).fg(Color::White);
    }

    let base = if app.color {
        Style::new().fg(byte_color(b))
    } else {
        Style::new()
    };
    if pane_active {
        base
    } else {
        base.add_modifier(Modifier::DIM)
    }
}

fn render_hex(f: &mut Frame, app: &App, area: Rect) {
    let hex_active = app.active_pane == ActivePane::Hex;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if hex_active {
            Style::new().fg(Color::Gray)
        } else {
            Style::new().fg(Color::DarkGray)
        });
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = (0..app.visible_rows)
        .map(|i| {
            let start = (app.scroll_row + i) * app.bytes_per_row;
            let mut spans = Vec::new();

            for col in 0..app.bytes_per_row {
                let idx = start + col;
                if col > 0 {
                    spans.push(Span::raw(" "));
                }
                if idx >= app.data.len() {
                    spans.push(Span::raw("  "));
                    continue;
                }

                let b = app.data[idx];
                // Pending nibble indicator
                if idx == app.cursor && hex_active && app.nibble == NibbleHalf::Low {
                    let hi = HEX_CHARS[app.pending_nibble as usize];
                    spans.push(Span::styled(
                        format!("{hi}_"),
                        Style::new().bg(Color::Yellow).fg(Color::Black),
                    ));
                } else {
                    spans.push(Span::styled(
                        format!("{:02x}", b),
                        byte_style(app, idx, ActivePane::Hex),
                    ));
                }
            }
            Line::from(spans)
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_ascii(f: &mut Frame, app: &App, area: Rect) {
    let ascii_active = app.active_pane == ActivePane::Ascii;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if ascii_active {
            Style::new().fg(Color::Gray)
        } else {
            Style::new().fg(Color::DarkGray)
        });
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = (0..app.visible_rows)
        .map(|i| {
            let start = (app.scroll_row + i) * app.bytes_per_row;
            let spans: Vec<Span> = (0..app.bytes_per_row)
                .map(|col| {
                    let idx = start + col;
                    if idx >= app.data.len() {
                        return Span::raw(" ");
                    }
                    let b = app.data[idx];
                    let ch = if (32..=126).contains(&b) {
                        b as char
                    } else {
                        '.'
                    };
                    Span::styled(ch.to_string(), byte_style(app, idx, ActivePane::Ascii))
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_hints(f: &mut Frame, app: &App, area: Rect) {
    let mut mode_str = match app.edit_mode {
        EditMode::Overwrite => "OVR",
        EditMode::Insert => "INS",
        EditMode::ReadOnly => "RO",
    };
    let sel_str = if app.sel_anchor.is_some() {
        let (lo, hi) = app.sel_range().unwrap();
        format!("  sel:{}", hi - lo + 1)
    } else {
        String::new()
    };

    if !sel_str.is_empty() {
        mode_str = "SEL"
    }

    let key = Style::new().reversed();
    let sep = Style::new();

    let line = Line::from(vec![
        Span::styled("^X", key),
        Span::styled(" Quit  ", sep),
        // Span::styled("^S", key),
        // Span::styled(" Save  ", sep),
        // Span::styled("^F", key),
        // Span::styled(" Find  ", sep),
        // Span::styled("^G", key),
        // Span::styled(" Goto  ", sep),
        // Span::styled("^E", key), // coming soon..
        Span::styled(" Toggle  ", sep),
        Span::styled("Tab", key),
        Span::styled(" Switch  ", sep),
        Span::styled(",./[]", key),
        Span::styled(" Resize  ", sep),
        Span::styled("^C", key),
        Span::styled(" Copy  ", sep),
        Span::styled("⇧+↕↔", key),
        Span::styled("/drag", sep),
        Span::styled(" Select  ", sep),
        Span::styled(format!("[{mode_str}]"), key),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
