//! The colours of the shelf's chrome, the bars across the top and the bottom, from the kit's
//! palette or a card's Config/theme.txt.

use std::sync::OnceLock;

use cartridge_store::Theme;

use crate::kit::palette::Palette;
use crate::sprite::SPRITE_PX;
use crate::status_bar::{LINK_OFF, LINK_ON};

/// The UI kit's bar across the bottom of the shelf: `kit::action::BAR` art pixels tall.
pub const BAR_H: f32 = (crate::kit::action::BAR * SPRITE_PX) as f32;

/// The card's Config/theme.txt, for the widgets that still draw in its greys rather than the
/// kit's palette. The top bar takes its colours from [`ChromeColours`] instead, handed to it
/// where it is drawn.
static THEME: OnceLock<Theme> = OnceLock::new();

pub fn set_theme(theme: Theme) {
    let _ = THEME.set(theme);
}

pub fn theme() -> &'static Theme {
    THEME.get_or_init(Theme::default)
}

fn rgb(c: [u8; 3]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        1.0,
    ]
}

pub fn housing() -> [f32; 4] {
    rgb(theme().housing)
}

pub fn edge() -> [f32; 4] {
    rgb(theme().edge)
}

/// The colours the shelf's chrome is drawn in: the bar across the top.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChromeColours {
    /// The top bar's body.
    pub housing: [f32; 4],
    /// The top bar's rule.
    pub edge: [f32; 4],
    /// What the top bar prints in: the clock, the battery and its figure, and what the
    /// handheld is connected to. Its faces are baked in it, so it is kept as bytes.
    pub ink: [u8; 3],
    /// What the top bar prints something that is off in: Wi-Fi, Bluetooth or SSH not
    /// connected.
    pub ink_off: [u8; 3],
}

impl ChromeColours {
    /// The chrome in the kit's colours, as flow.html draws the shelf: the top bar in the well
    /// colour, ruled in the edge colour. It prints in cream, as flow.html's does, and what is
    /// off in the plate colour, as a network row draws the Wi-Fi bars a signal does not reach.
    pub fn of(pal: &Palette) -> Self {
        ChromeColours {
            housing: rgb(pal.well),
            edge: rgb(pal.edge),
            ink: pal.cream,
            ink_off: pal.plate,
        }
    }

    /// The chrome a card's Config/theme.txt sets, whatever the palette. A theme names no ink,
    /// so the top bar keeps the white and grey it was printed in before it took the palette.
    pub fn themed(theme: &Theme) -> Self {
        ChromeColours {
            housing: rgb(theme.housing),
            edge: rgb(theme.edge),
            ink: LINK_ON,
            ink_off: LINK_OFF,
        }
    }

    /// What the top bar prints an item in, `on` or off.
    pub fn link_ink(&self, on: bool) -> [u8; 3] {
        if on {
            self.ink
        } else {
            self.ink_off
        }
    }
}

/// The greys the chrome was drawn in before it took the kit's palette, which a theme.txt
/// still starts from.
impl Default for ChromeColours {
    fn default() -> Self {
        ChromeColours::themed(&Theme::default())
    }
}
