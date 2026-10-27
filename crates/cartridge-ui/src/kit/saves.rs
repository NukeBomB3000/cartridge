//! Which save state the switcher shows, as assets/ui/components/cartridge-saves.js: a panel titled
//! SAVE STATES over the game's screenshot, how long ago the save shown was made in its well, and
//! under that a dot for each save the cart has, newest first, the one shown lit.
//!
//! The panel's tab is only as wide as its title (cartridge-panel's `--cartridge-tab-width: 0`), and the
//! panel as wide as the title, the age or the dots need ([`saves_width`]), or as wide as it is
//! given: the switcher gives it the room of the widest age it has, so it holds still as the
//! selection moves. The age is on the well's line of 10 in cream, in line with the title; the dots
//! a row past that line, two clear of its capitals where a descender reaches, each 5 art pixels
//! round and 3 apart, the one shown in cream and the rest in the plate colour, what is off on a
//! row; and the well shows as much under them as over the capitals, so it is `SAVES_H` tall.

use super::layout::PANEL_CHROME_H;
use super::palette::Palette;
use super::panel::{panel_face_at, tab_width_at, HEADER};
use super::pixel::{ink, letters, paint, text_width, ADVANCE, LINE};
use crate::plate::UndoFace;

/// The panel's title.
pub const TITLE: &str = "SAVE STATES";

/// A dot, 5 pixels round: '#' is drawn lit or not.
#[rustfmt::skip]
const DOT: [&str; 5] = [
    ".###.",
    "#####",
    "#####",
    "#####",
    ".###.",
];

/// How wide a dot is, and how far apart two are.
pub const DOT_W: u32 = 5;
const DOT_GAP: u32 = 3;

/// How far down the well's inside the dots start: a row past the age's line, two rows clear of
/// its capitals. And the rows of well under them, which with the well's own room below make as
/// many as there are over the capitals.
pub const DOTS_TOP: u32 = LINE + 1;
const UNDER_DOTS: u32 = 2;

/// How tall the panel is: its chrome round a well holding the age's line and the dots.
pub const SAVES_H: u32 = DOTS_TOP + DOT.len() as u32 + UNDER_DOTS + PANEL_CHROME_H;

/// What a panel's chrome takes across beside its well, both sides together: its stripe, the
/// frame line and the well's shadow line, and the well's room either side.
const WELL_SIDES: u32 = 2 * (1 + 1 + 1 + 3);

/// How wide `count` dots are, from the first one's left to the last one's right.
pub fn dots_width(count: usize) -> u32 {
    count.max(1) as u32 * (DOT_W + DOT_GAP) - DOT_GAP
}

/// How wide the panel is to hold `age` and `count` dots: its title's tab and the slant after it
/// inside its stripe, or the age or the dots inside the well's chrome, whichever is wider.
pub fn saves_width(age: &str, count: usize) -> u32 {
    let header = 1 + tab_width_at(TITLE, 0) + HEADER + 1;
    header.max(WELL_SIDES + text_width(age).max(dots_width(count)))
}

/// `count` dots in a row as sprite rows, the one at `at` lit.
fn dots(count: usize, at: usize) -> Vec<String> {
    DOT.iter()
        .map(|row| {
            (0..count)
                .map(|i| row.replace('#', if i == at { "c" } else { "p" }))
                .collect::<Vec<_>>()
                .join(&".".repeat(DOT_GAP as usize))
        })
        .collect()
}

/// The panel `w` art pixels wide saying the save shown is `age` old, the one at `at` of `count`
/// (at least one; 0 the newest). Give it `saves_width` or more.
pub fn saves_face(w: u32, age: &str, count: usize, at: usize, pal: &Palette) -> UndoFace {
    let count = count.max(1);
    let mut panel = panel_face_at(w, SAVES_H, TITLE, true, 0, pal);
    let [x, y, _, _] = panel.well.map(|v| v as i32);
    ink(&mut panel.face, &letters(age, ADVANCE), x, y, pal.cream);
    let dots = dots(count, at.min(count - 1));
    paint(&mut panel.face, &dots, x, y + DOTS_TOP as i32, pal);
    panel.face
}
