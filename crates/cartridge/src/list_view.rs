//! What a settings list in the quick menu's style shows, for its faces to be made from: the
//! Wi-Fi screen and the Labels screen are both drawn from one.

use cartridge_ui::Keyboard;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegendHint {
    Back,
    Change,
    Join,
    Forget,
    SignIn,
    SignOut,
    Start,
    Stop,
    Open,
}

impl LegendHint {
    pub const ALL: [LegendHint; 9] = [
        LegendHint::Back,
        LegendHint::Change,
        LegendHint::Join,
        LegendHint::Forget,
        LegendHint::SignIn,
        LegendHint::SignOut,
        LegendHint::Start,
        LegendHint::Stop,
        LegendHint::Open,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowLabel {
    Setting(&'static str),
    /// A name that is not cartridge's to set in capitals, such as a network's.
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowValue {
    /// On or Off, changed with left and right.
    Setting(bool),
    /// One of a few named choices, changed with left and right.
    Choice(&'static str),
    Said(&'static str),
    /// A name kept in its own case, such as an account's.
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewRow {
    pub label: RowLabel,
    pub value: Option<RowValue>,
    /// A row with nothing to change here, drawn faint and passed over by the cursor.
    pub dim: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListView {
    pub rows: Vec<ViewRow>,
    pub lit: Option<usize>,
    pub status: Vec<String>,
    pub legend: Vec<LegendHint>,
}

/// The on-screen keyboard while a screen has it up, and what it is asking for.
pub struct TypingView<'a> {
    pub prompt: String,
    pub keys: &'a Keyboard,
    /// What was typed was refused.
    pub refused: bool,
}
