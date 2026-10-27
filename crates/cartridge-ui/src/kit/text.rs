//! Plain lines of Pixel Code for what a panel holds that is not a row, as flow.html sets them:
//! SSH's "SFTP uses the same login", the Labels note, the Controls panel's two columns of buttons
//! and what they do. One face pixel to an art pixel, in the case they are given, each line 10
//! art pixels below the one before with its capitals on rows 2 to 8, on whole pixels.
//!
//! The arrows the Controls panel names buttons with, ◀ ▶ ▲ ▼, are Pixel Code's own: the full
//! face the crate embeds has them, and draws them pixel for pixel as flow/art.js copies them
//! (the storybook's web font is cut to Latin-1 and has not).
//!
//! Where flow.html puts them: a note in a panel's well starts 3 pixels in from the well's
//! inside (`PanelFace::well`'s x + 3) and 4 below the row over it; the Controls panel's lines
//! are 11 apart (a line and a pixel), the buttons in the frame's bright colour, what they do in
//! cream, the column headings in the dim one.

use super::pixel::{blank, ink, letters, text_width, ACCENT, ADVANCE, CAP, DESCENT, LINE};
use crate::plate::UndoFace;

/// `text` in Pixel Code in `colour` on a clear face: as wide as its longest line (6 art pixels
/// a letter, the last one's blank column included) and, one line on another for each '\n', as
/// tall as its lines and the 11th row of the last, where a descender may still reach. Set the
/// face's top at the first line's top.
pub fn text_face(text: &str, colour: [u8; 3]) -> UndoFace {
    let lines: Vec<&str> = text.split('\n').collect();
    let w = lines.iter().map(|l| text_width(l)).max().unwrap_or(0);
    let rows = (ACCENT + CAP + DESCENT) as u32;
    let mut face = blank(w.max(1), (lines.len() as u32 - 1) * LINE + rows);
    for (i, line) in lines.iter().enumerate() {
        let y = i as i32 * LINE as i32;
        ink(&mut face, &letters(line, ADVANCE), 0, y, colour);
    }
    face
}

/// How many art pixels tall a line of `text_face` is: the capitals' rows with the accents' over
/// them and the descenders' under them.
pub const WORD_H: u32 = (ACCENT + CAP + DESCENT) as u32;

/// The clear column every letter keeps on its right inside its `ADVANCE`, the last letter's
/// included: a `text_face`'s ink ends this far short of its right edge.
pub const AIR: u32 = 1;
