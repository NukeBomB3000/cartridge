//! The window of the Mega Man Zero menus, as assets/ui/components/cartridge-modal.js: a frame, and
//! the panels it stacks top to bottom, sharing its height.

use super::palette::Palette;
use super::pixel::{blank, mirror, nine_slice};
use crate::plate::UndoFace;

/// The frame's top-left corner, drawn pixel by pixel and mirrored into the other three: a dark
/// outline, a bright line, a stripe, then the frame colour the panels sit on. The last row and
/// column are the straight edge between corners.
#[rustfmt::skip]
const CORNER: [&str; 5] = [
    "...kk",
    ".kkHH",
    ".kHss",
    "kHsAA",
    "kHsAA",
];

/// How deep the frame is: the corner's slice, everything but its middle row and column.
pub const CUT: u32 = CORNER.len() as u32 - 1;

/// How far the panels reach up into the frame, so the first one's stripe lands on the frame's
/// own and its header starts right under it, as in the game.
const REACH: u32 = 2;

/// The modal's frame, `w` by `h` art pixels, its middle filled with the frame colour.
pub fn modal_face(w: u32, h: u32, pal: &Palette) -> UndoFace {
    let mut face = blank(w, h);
    let rect = [0, 0, w as i32, h as i32];
    nine_slice(&mut face, &mirror(&CORNER), rect, [CUT; 4], [CUT; 4], pal);
    face
}

/// Where the panels of a `w` by `h` modal go, as `[x, y, w, h]` in its art pixels, one for each
/// of `weights`: they share its height by those flex-grows, the first two pixels up into the
/// frame and each after it a pixel over the one before, so two panels share one stripe.
///
/// A share that is not a whole pixel is laid out as Chrome lays it out, in 64ths of a pixel,
/// and each edge rounded to the nearest pixel, a half up. Each panel is then a `panel_face` of
/// its rect's size: its edges round together with its top or its bottom.
pub fn modal_layout(w: u32, h: u32, weights: &[u32]) -> Vec<[u32; 4]> {
    let content = i64::from(h.saturating_sub(2 * CUT) + REACH) * 64;
    let overlaps = weights.len().saturating_sub(1) as i64 * 64;
    let free = content + overlaps;
    let total: i64 = weights.iter().map(|&g| i64::from(g)).sum();
    let snap = |at: i64| ((at + 32) >> 6) as u32;
    let mut top = i64::from(CUT - REACH) * 64;
    weights
        .iter()
        .map(|&grow| {
            let size = match total {
                0 => 0,
                // Chrome's LayoutUnit::FromFloatRound: to the nearest 64th, a half away from 0
                _ => (2 * free * i64::from(grow) + total) / (2 * total),
            };
            let (y, bottom) = (snap(top), snap(top + size));
            top += size - 64;
            [CUT, y, w.saturating_sub(2 * CUT), bottom - y]
        })
        .collect()
}
