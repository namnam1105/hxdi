use crate::tui::types::*;

pub struct App {
    pub data: Vec<u8>,
    pub original: Vec<u8>,
    pub file_name: Option<String>,

    pub scroll_row: usize,
    pub visible_rows: usize,
    pub bytes_per_row: usize,
    pub offset_digits: u16,

    pub cursor: usize,
    pub nibble: NibbleHalf,
    pub pending_nibble: u8,

    pub edit_mode: EditMode,
    pub active_pane: ActivePane,

    pub dialog: Dialog,
    pub status_msg: Option<String>,
    pub status_msg_until: Option<std::time::Instant>,

    // display flags (from CLI args)
    pub color: bool,
    pub show_header: bool,
    pub show_offsets: bool,
    pub show_hex: bool,
    pub show_ascii: bool,

    // selection
    pub sel_anchor: Option<usize>,

    // resize state (updated by render each frame)
    pub bpr_override: Option<usize>,
    pub offset_extra: i16,
    pub dragging: bool,
    pub drag_origin: Option<usize>, // byte index where mouse-down landed (for drag-select)
    pub sep_col: u16,
    pub offset_w: u16,
    pub frame_width: u16,

    // mouse hit-test bounds (set each frame by render)
    pub editor_content_y: u16,
    pub hex_content_x: u16,
    pub hex_content_w: u16,
    pub ascii_content_x: u16,
    pub ascii_content_w: u16,
}

impl App {
    pub fn new(
        data: Vec<u8>,
        file_name: Option<String>,
        read_only: bool,
        color: bool,
        show_header: bool,
        show_offsets: bool,
        show_hex: bool,
        show_ascii: bool,
    ) -> Self {
        let original = data.clone();
        App {
            data,
            original,
            file_name,
            scroll_row: 0,
            visible_rows: 0,
            bytes_per_row: 16,
            offset_digits: 4,
            cursor: 0,
            nibble: NibbleHalf::High,
            pending_nibble: 0,
            edit_mode: if read_only { EditMode::ReadOnly } else { EditMode::Overwrite },
            active_pane: ActivePane::Hex,
            dialog: Dialog::None,
            status_msg: None,
            status_msg_until: None,
            color,
            show_header,
            show_offsets,
            show_hex,
            show_ascii,
            sel_anchor: None,
            bpr_override: None,
            offset_extra: 0,
            dragging: false,
            drag_origin: None,
            sep_col: 0,
            offset_w: 0,
            frame_width: 0,
            editor_content_y: 0,
            hex_content_x: 0,
            hex_content_w: 0,
            ascii_content_x: 0,
            ascii_content_w: 0,
        }
    }

    pub fn is_dirty(&self) -> bool { self.data != self.original }
    pub fn cursor_row(&self) -> usize { self.cursor / self.bytes_per_row }
    pub fn cursor_col(&self) -> usize { self.cursor % self.bytes_per_row }
    pub fn total_rows(&self) -> usize {
        self.data.len().saturating_sub(1) / self.bytes_per_row + 1
    }

    pub fn sel_range(&self) -> Option<(usize, usize)> {
        self.sel_anchor.map(|a| {
            let lo = a.min(self.cursor);
            let hi = a.max(self.cursor);
            (lo, hi)
        })
    }
}
