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
