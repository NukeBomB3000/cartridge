//! What a game is doing that lasts, in the corner over it, as
//! assets/ui/components/cartridge-badge.js: the bottom bar's key cap holding a 7x7 sprite and, after
//! it, a word. Fast forward shows its sprite and how fast, "×4"; a link to another handheld its
//! link, or the link broken once it is lost.
//!
//! It is 9 art pixels tall, as tall as a key cap in the bar: a row above the capitals and a row
//! below. The sprite starts two pixels in, the cap's end column and one more, the word 3 pixels
//! after it, and on the right the word's own blank pixel and the end column make the same two. Lit,
//! it is drawn the way a lit row's band is: the tab colour with a frame line round it.

use super::gauge::GaugeIcon;
use super::palette::Palette;
use super::pixel::{blank, ink, letters, mirror, nine_slice, text_width, ACCENT, ADVANCE};
use crate::plate::UndoFace;

/// The cap as a 9-slice: dark, its four corner pixels left out, as the bar's key cap.
#[rustfmt::skip]
const CAP: [&str; 2] = [
    ".k",
    "kk",
];

/// Lit: a lit row's band, the frame line round the tab colour, its corner pixels left out.
#[rustfmt::skip]
const BAND: [&str; 2] = [
    ".A",
    "At",
];

/// How tall a badge is.
pub const BADGE_H: u32 = 9;

/// The room between the sprite and the word.
const GAP: u32 = 3;

/// A sprite's side: as tall as the capitals.
const ICON: u32 = 7;

/// The sprite a badge holds, drawn in cream.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BadgeIcon {
    FastForward,
    /// A link to another handheld.
    Link,
    /// A link that was lost.
    Unlinked,
}

impl BadgeIcon {
    /// The sprite, rows of '#' for the ink.
    #[rustfmt::skip]
    pub fn rows(self) -> [&'static str; ICON as usize] {
        match self {
            BadgeIcon::FastForward => GaugeIcon::FastForward.rows(),
            BadgeIcon::Link => [
                ".......",
                ".##.##.",
                "#.....#",
                "#.###.#",
                "#.....#",
                ".##.##.",
                ".......",
            ],
            BadgeIcon::Unlinked => [
                "...#...",
                ".##.##.",
                "#.....#",
                "#.....#",
                "#.....#",
                ".##.##.",
                "...#...",
            ],
        }
    }
}

/// How wide a badge with `text` after its sprite is: the cap's end column and a pixel, the
/// sprite, the word with the room before it (its last column its own blank pixel) or, with no
/// word, a pixel, then the other end column.
pub fn badge_width(text: &str) -> u32 {
    let after = match text {
        "" => 1,
        text => GAP + text_width(text),
    };
    1 + 1 + ICON + after + 1
}

/// A badge holding `icon` and then `text`, lit or not.
pub fn badge_face(icon: BadgeIcon, text: &str, lit: bool, pal: &Palette) -> UndoFace {
    let w = badge_width(text);
    let mut face = blank(w, BADGE_H);
    let cap = mirror(if lit { &BAND } else { &CAP });
    let all = [0, 0, w as i32, BADGE_H as i32];
    nine_slice(&mut face, &cap, all, [1; 4], [1; 4], pal);
    ink(&mut face, &icon.rows(), 2, 1, pal.cream);
    if !text.is_empty() {
        let x = 2 + (ICON + GAP) as i32;
        let line = letters(text, ADVANCE);
        // the word's capitals on the sprite's rows, nothing of its line above or below them
        ink(
            &mut face,
            &line[ACCENT..ACCENT + ICON as usize],
            x,
            1,
            pal.cream,
        );
    }
    face
}
