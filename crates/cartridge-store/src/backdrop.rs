//! What the shelf is drawn over: a pattern in one of a few colours, still or drifting, or a
//! picture of the player's own from Wallpapers/.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackdropStyle {
    /// Lines crossing, like squared paper.
    Grid,
    /// Raised square plates, lit from the top left like a 16-bit floor.
    Plates,
    /// Dotted lines, a cross where they meet.
    Dotted,
    /// The grid's squares shaded from the top left in an ordered dither.
    Dither,
    /// The plates riveted at each corner.
    Rivets,
    /// Square pixels scattered like a 1-bit picture.
    Bits,
    /// The colour alone.
    Solid,
    /// A .png from Wallpapers/ on the card.
    Wallpaper,
}

impl BackdropStyle {
    pub const ALL: [BackdropStyle; 8] = [
        BackdropStyle::Grid,
        BackdropStyle::Plates,
        BackdropStyle::Dotted,
        BackdropStyle::Dither,
        BackdropStyle::Rivets,
        BackdropStyle::Bits,
        BackdropStyle::Solid,
        BackdropStyle::Wallpaper,
    ];

    pub fn key(self) -> &'static str {
        match self {
            BackdropStyle::Grid => "grid",
            BackdropStyle::Plates => "plates",
            BackdropStyle::Dotted => "dotted",
            BackdropStyle::Dither => "dither",
            BackdropStyle::Rivets => "rivets",
            BackdropStyle::Bits => "bits",
            BackdropStyle::Solid => "solid",
            BackdropStyle::Wallpaper => "wallpaper",
        }
    }

    pub fn from_key(key: &str) -> Option<BackdropStyle> {
        Self::ALL.into_iter().find(|s| s.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            BackdropStyle::Grid => "Grid",
            BackdropStyle::Plates => "Plates",
            BackdropStyle::Dotted => "Dotted",
            BackdropStyle::Dither => "Dither",
            BackdropStyle::Rivets => "Rivets",
            BackdropStyle::Bits => "Bits",
            BackdropStyle::Solid => "Solid",
            BackdropStyle::Wallpaper => "Wallpaper",
        }
    }

    /// A pattern that can drift.
    pub fn moves(self) -> bool {
        !matches!(self, BackdropStyle::Solid | BackdropStyle::Wallpaper)
    }

    /// Drawn in one of the palette's colours.
    pub fn coloured(self) -> bool {
        self != BackdropStyle::Wallpaper
    }
}

/// The colours a pattern comes in, by the name cartridge.state keeps, darkest first. cartridge-ui holds
/// what each looks like, in this order.
pub const BACKDROP_COLOURS: [&str; 8] = [
    "night", "slate", "navy", "teal", "green", "plum", "crimson", "amber",
];

pub fn colour_index(key: &str) -> Option<u8> {
    BACKDROP_COLOURS
        .iter()
        .position(|c| *c == key)
        .map(|i| i as u8)
}
