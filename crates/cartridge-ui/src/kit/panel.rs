//! One titled section inside a modal, like BUTTON SETTING and ATTACK MODE, as
//! assets/ui/components/cartridge-panel.js: a dark title tab with a slanted end, the bright strip
//! beside it, and a well below for content.
//!
//! Moving the cursor from one panel to another re-bakes nothing: bake each panel once with its
//! title dim (`active` false), and draw the active one's `title_face` over its title at
//! `PanelFace::title`. The two titles ink the very same pixels, so the light one covers the dim
//! one exactly.

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

/// How tall the header is in art pixels, its bottom stripe included, and the rows the tab and
/// its slant fill above that stripe.
pub const HEADER: u32 = 13;
const ROWS: u32 = HEADER - 1;

/// How wide the title tab is unless its title needs more (`--cartridge-tab-width`'s default).
pub const TAB_W: u32 = 121;

/// The room inside the tab on either side of its title.
const TAB_PAD: u32 = 4;

/// Where a panel's title starts, in its art pixels: inside the panel's stripe, the tab's frame
/// line and its padding across; inside the stripe and the tab's frame line down, on a line of
/// `LINE` whose rows 2 to 8 its capitals fill.
pub const TITLE_X: u32 = 1 + 1 + TAB_PAD;
pub const TITLE_Y: u32 = 1 + 1;

/// The room inside the well around what it holds: above and below, then either side.
const WELL_PAD: [u32; 2] = [2, 3];

/// A panel's chrome with its title baked in.
pub struct PanelFace {
    pub face: UndoFace,
    /// The well's inside, `[x, y, w, h]` in the face's art pixels: where a panel's rows go.
    pub well: [u32; 4],
    /// The title's `title_face`, `[x, y, w, h]` in the face's art pixels.
    pub title: [u32; 4],
}

/// The tab's slanted end: one pixel further right each row, so it leans 45 degrees like the
/// game's. Row by row, the tab's fill (its top and bottom rows in the frame colour, as its
/// outline), a frame pixel, a stripe pixel, then nothing, where the strip shows.
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

/// How wide the tab is with `title` in it: `TAB_W`, or as wide as the title and its room.
pub fn tab_width(title: &str) -> u32 {
    tab_width_at(title, TAB_W)
}

/// How wide the tab is with `title` in it when it is given `min`, as cartridge-panel is given
/// `--cartridge-tab-width`: `min`, or as wide as the title and its room; 0 keeps it to the title.
pub fn tab_width_at(title: &str, min: u32) -> u32 {
    min.max(1 + 2 * TAB_PAD + text_width(title))
}

/// `title` alone, as a panel's tab shows it: Pixel Code in the active panel's cream or the
/// others' dim, on a clear face `PanelFace::title`'s size.
pub fn title_face(title: &str, active: bool, pal: &Palette) -> UndoFace {
    let rows = letters(title, ADVANCE);
    let mut face = blank(rows[0].len() as u32, rows.len() as u32);
    ink(
        &mut face,
        &rows,
        0,
        0,
        if active { pal.cream } else { pal.dim },
    );
    face
}

/// A panel `w` by `h` art pixels titled `title`, its title light when `active` (the section the
/// cursor is in) and dimmed when not.
pub fn panel_face(w: u32, h: u32, title: &str, active: bool, pal: &Palette) -> PanelFace {
    panel_face_at(w, h, title, active, TAB_W, pal)
}

/// The same panel, its tab given `tab_min` as cartridge-panel is given `--cartridge-tab-width` (see
/// [`tab_width_at`]).
pub fn panel_face_at(
    w: u32,
    h: u32,
    title: &str,
    active: bool,
    tab_min: u32,
    pal: &Palette,
) -> PanelFace {
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
    fill(&mut face, [1, 1, wi - 2, ROWS as i32], pal.frame_hi);
    fill(&mut face, [1, HEADER as i32, wi - 2, 1], pal.stripe);

    // the tab, outlined in the frame colour on every side but its slanted one
    let tab = tab_width_at(title, tab_min) as i32;
    let rows = ROWS as i32;
    fill(&mut face, [1, 1, tab, rows], pal.tab);
    fill(&mut face, [1, 1, tab, 1], pal.frame);
    fill(&mut face, [1, rows, tab, 1], pal.frame);
    fill(&mut face, [1, 1, 1, rows], pal.frame);
    paint(&mut face, &slant(), 1 + tab, 1, pal);

    // under the header: a frame line, then the well's shadow line round the well
    let body = [2, 2 + HEADER as i32, wi - 4, hi - HEADER as i32 - 4];
    nine_slice(&mut face, &mirror(&WELL), body, [1; 4], [1; 4], pal);

    let title_rows = letters(title, ADVANCE);
    let colour = if active { pal.cream } else { pal.dim };
    ink(
        &mut face,
        &title_rows,
        TITLE_X as i32,
        TITLE_Y as i32,
        colour,
    );

    let [pad_y, pad_x] = WELL_PAD;
    let well = [
        body[0] as u32 + 1 + pad_x,
        body[1] as u32 + 1 + pad_y,
        (body[2].max(0) as u32).saturating_sub(2 + 2 * pad_x),
        (body[3].max(0) as u32).saturating_sub(2 + 2 * pad_y),
    ];
    let title = [
        TITLE_X,
        TITLE_Y,
        title_rows[0].len() as u32,
        title_rows.len() as u32,
    ];
    PanelFace { face, well, title }
}
