//! The font the bar's actions write their big words in, as Settings > Theme > Font picks.

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum Font {
    /// Each letter drawn again a pixel to its right, as the bar always was.
    #[default]
    Bold,
    /// The letters in one-pixel strokes, so the words are narrower.
    Thin,
}

impl Font {
    pub const ALL: [Font; 2] = [Font::Bold, Font::Thin];

    pub fn key(self) -> &'static str {
        match self {
            Font::Bold => "bold",
            Font::Thin => "thin",
        }
    }

    pub fn from_key(key: &str) -> Option<Font> {
        Self::ALL.into_iter().find(|f| f.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            Font::Bold => "Bold",
            Font::Thin => "Thin",
        }
    }
}
