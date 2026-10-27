//! A word on what just happened, as assets/ui/components/cartridge-toast.js: a lone panel, active,
//! its title tab naming what it is about and one line in its well. It comes and goes on its own
//! over a game, the shelf or a menu: SAVE STATE "State saved", LINK "Link was ended".
//!
//! It is drawn as panel.rs draws a panel, but as wide as the wider of its header and its line:
//! the tab hugs its title, 4 pixels either side, its slanted end after it, and the strip takes
//! whatever room the line leaves. The line starts in line with the title, 3 pixels into the well,
//! and its capitals have 4 rows of well above them and 4 below.

use super::palette::Palette;
use super::pixel::{blank, fill, ink, letters, mirror, nine_slice, paint, text_width, ADVANCE};
use crate::plate::UndoFace;

/// The panel's corner: its stripe, with the corner pixel left out, around the frame colour.
#[rustfmt::skip]
const PANEL: [&str; 2] = [
    ".s",
    "sA",
];

/// The well's corner: its shadow line, the same way, around the well.
#[rustfmt::skip]
const WELL: [&str; 2] = [
    ".e",
    "ew",
];

/// How tall the header is, its bottom stripe included, and the rows the tab and its slant fill
/// above that stripe.
const HEADER: u32 = 13;
const ROWS: u32 = HEADER - 1;

/// The room inside the tab on either side of its title.
const TAB_PAD: u32 = 4;

/// The room inside the well round the line: above it, under it, and either side.
const WELL_TOP: u32 = 2;
const WELL_BOTTOM: u32 = 3;
const WELL_SIDE: u32 = 3;

/// A line of text, its capitals on rows 2 to 8.
const LINE: u32 = 10;

/// How tall a toast is: the panel's stripe, the header, a frame line and the well's shadow line
/// above the well and below it, and the well round its line.
pub const TOAST_H: u32 = 1 + HEADER + 2 + WELL_TOP + LINE + WELL_BOTTOM + 2 + 1;

/// The tab's slanted end, as a panel's: one pixel further right each row, its top and bottom
/// rows in the frame colour as the tab's outline, then a frame pixel and a stripe pixel, then
/// nothing, where the strip shows.
fn slant() -> Vec<String> {
    (0..ROWS as usize)
        .map(|y| {
            let fill = if y == 0 || y == ROWS as usize - 1 {
                "A"
            } else {
                "t"
            };
            fill.repeat(y) + "As" + &".".repeat(HEADER as usize - y - 2)
        })
        .collect()
}

/// How wide the tab is round `label`: its frame line on the left and the room either side.
fn tab_width(label: &str) -> u32 {
    1 + 2 * TAB_PAD + text_width(label)
}

/// How wide a toast saying `text` under `label` is: its header with no strip, or its line with
/// the well's room, shadow line and frame line either side, whichever is wider.
pub fn toast_width(label: &str, text: &str) -> u32 {
    let header = 1 + tab_width(label) + HEADER + 1;
    let line = 1 + 1 + 1 + WELL_SIDE + text_width(text) + WELL_SIDE + 1 + 1 + 1;
    header.max(line)
}

/// The toast titled `label`, in capitals whatever case it is given in, saying `text` in its own
/// case: the title and the line in cream.
pub fn toast_face(label: &str, text: &str, pal: &Palette) -> UndoFace {
    let label = label.to_uppercase();
    let (w, h) = (toast_width(&label, text), TOAST_H);
    let mut face = blank(w, h);
    let (wi, hi) = (w as i32, h as i32);
    nine_slice(
        &mut face,
        &mirror(&PANEL),
        [0, 0, wi, hi],
        [1; 4],
        [1; 4],
        pal,
    );

    // the header: the strip's bright colour over its bottom stripe, inside the panel's stripe
    let rows = ROWS as i32;
    fill(&mut face, [1, 1, wi - 2, rows], pal.frame_hi);
    fill(&mut face, [1, HEADER as i32, wi - 2, 1], pal.stripe);

    // the tab, outlined in the frame colour on every side but its slanted one
    let tab = tab_width(&label) as i32;
    fill(&mut face, [1, 1, tab, rows], pal.tab);
    fill(&mut face, [1, 1, tab, 1], pal.frame);
    fill(&mut face, [1, rows, tab, 1], pal.frame);
    fill(&mut face, [1, 1, 1, rows], pal.frame);
    paint(&mut face, &slant(), 1 + tab, 1, pal);
    let title_x = 1 + 1 + TAB_PAD as i32;
    ink(&mut face, &letters(&label, ADVANCE), title_x, 2, pal.cream);

    // under the header: a frame line, then the well's shadow line round the well
    let body = [2, 2 + HEADER as i32, wi - 4, hi - HEADER as i32 - 4];
    nine_slice(&mut face, &mirror(&WELL), body, [1; 4], [1; 4], pal);
    let (x, y) = (
        body[0] + 1 + WELL_SIDE as i32,
        body[1] + 1 + WELL_TOP as i32,
    );
    ink(&mut face, &letters(text, ADVANCE), x, y, pal.cream);
    face
}
