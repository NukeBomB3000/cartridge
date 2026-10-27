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
    /// A Game Boy's screen: its dot matrix, each dot a square with a dark seam round it.
    GameBoy,
    /// The DS Lite's menu: fine lines across, lighter toward the top.
    Ds,
    /// The 3DS's: broad stripes at 45 degrees.
    ThreeDs,
    /// The PSP's: ribbons waving across a fade from the top.
    Psp,
    /// The Switch's: a fade from the top and a band behind the row of games.
    Switch,
    /// The colour alone.
    Solid,
    /// A .png from Wallpapers/ on the card.
    Wallpaper,
}

impl BackdropStyle {
    pub const ALL: [BackdropStyle; 13] = [
        BackdropStyle::Grid,
        BackdropStyle::Plates,
        BackdropStyle::Dotted,
        BackdropStyle::Dither,
        BackdropStyle::Rivets,
        BackdropStyle::Bits,
        BackdropStyle::GameBoy,
        BackdropStyle::Ds,
        BackdropStyle::ThreeDs,
        BackdropStyle::Psp,
        BackdropStyle::Switch,
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
            BackdropStyle::GameBoy => "gameboy",
            BackdropStyle::Ds => "ds",
            BackdropStyle::ThreeDs => "3ds",
            BackdropStyle::Psp => "psp",
            BackdropStyle::Switch => "switch",
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
            BackdropStyle::GameBoy => "Game Boy",
            BackdropStyle::Ds => "DS",
            BackdropStyle::ThreeDs => "3DS",
            BackdropStyle::Psp => "PSP",
            BackdropStyle::Switch => "Switch",
            BackdropStyle::Solid => "Solid",
            BackdropStyle::Wallpaper => "Wallpaper",
        }
    }

    /// A pattern that can drift: one that repeats across. The DS's lines, the PSP's waves and
    /// the Switch's band stand still.
    pub fn moves(self) -> bool {
        self.patterned()
            && !matches!(
                self,
                BackdropStyle::Ds | BackdropStyle::Psp | BackdropStyle::Switch
            )
    }

    /// Drawn from a picture of a pattern, rather than the colour alone or the card's picture.
    pub fn patterned(self) -> bool {
        !matches!(self, BackdropStyle::Solid | BackdropStyle::Wallpaper)
    }

    /// Drawn in one of the palette's colours.
    pub fn coloured(self) -> bool {
        self != BackdropStyle::Wallpaper
    }
}

/// The colours a pattern comes in, by the name cartridge.state keeps, Night first, the other neutrals, then round the colour wheel. cartridge-ui holds
/// what each looks like, in this order.
pub const BACKDROP_COLOURS: [&str; 17] = [
    "night", "black", "slate", "crimson", "coral", "brown", "amber", "olive", "green", "mint",
    "teal", "sky", "navy", "indigo", "plum", "wine", "rose",
];

/// Night, the colour every card began in: near black, a breath of blue in it.
pub const NIGHT: u8 = 0;

pub fn colour_index(key: &str) -> Option<u8> {
    BACKDROP_COLOURS
        .iter()
        .position(|c| *c == key)
        .map(|i| i as u8)
}
