//! A screen's keys on one line of the kit, on a ground of their own, as
//! assets/ui/components/cartridge-hints.js: a title tab naming what they are for, then a well
//! holding the keys, each its key in a small dark cap and then the word, drawn as the bottom
//! bar draws its hints (bar.rs). It is for where the bar's hints would sit on a picture: the
//! save-state switcher stands SAVE, Y DELETE and, while the last save or load can still be
//! undone, X UNDO SAVE or X UNDO LOAD on one over the game's screenshot, just above the bar.
//!
//! It is a gauge (gauge.rs) with the keys in its well for the cells: 15 art pixels tall, as tall
//! as the bar; a panel's frame line round it, the tab and its slant, and on the strip's bright
//! colour the well, the keys 2 pixels inside its shadow line and 2 clear after their last ink.
//! Their capitals stand on the title's rows, so each key's cap, a row above and below them,
//! fills the well's inside from top to bottom. Given more room than that, the well takes it, the
//! keys at its left.

use super::bar::{draw_hints, hints_width, Hint};
use super::gauge::{chrome_width, gauge_chrome, well_width, well_x, CAPS_TOP, GAUGE_H, WELL_PAD};
use super::palette::Palette;
use super::pixel::ACCENT;
use crate::plate::UndoFace;

/// How tall the line is: a gauge's height, the bottom bar's.
pub const HINT_LINE_H: u32 = GAUGE_H;

/// How wide the line titled `label` is round `hints`: a gauge's chrome round a well holding
/// them from their first ink to their last.
pub fn hint_line_width(label: &str, hints: &[Hint]) -> u32 {
    chrome_width(label, well_width(hints_width(hints)))
}

/// The line titled `label` holding `hints`, `w` art pixels wide, or `hint_line_width` if that is
/// wider: the keys from the well's left, the title, each cap's letters and each word in cream.
/// `label` is drawn in capitals whatever case it is given in.
pub fn hint_line_face(w: u32, label: &str, hints: &[Hint], pal: &Palette) -> UndoFace {
    let w = w.max(hint_line_width(label, hints));
    let mut face = gauge_chrome(label, w, pal);
    // the keys' line starts ACCENT rows above their capitals, which are the title's
    let x = well_x(label) + 1 + WELL_PAD;
    let y = CAPS_TOP as i32 - ACCENT as i32;
    draw_hints(&mut face, hints, x as i32, y, pal);
    face
}
