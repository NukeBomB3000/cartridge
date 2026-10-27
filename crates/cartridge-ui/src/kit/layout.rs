//! Where the kit's windows go on the screen, as assets/ui/flow.html places them: a modal fills
//! the page above the bottom bar but for a margin, and a lone panel sits in the middle of it.
//! Everything here is in art pixels (two screen pixels each, SPRITE_PX), so whatever is placed
//! from it lands on whole art pixels.

use crate::draw::{out_h, out_w};
use crate::sprite::SPRITE_PX;

/// The bottom bar's height in art pixels; the page the menus sit in ends on its top.
pub const BAR_U: u32 = super::action::BAR;

/// The modal's margin inside the page: from the top, either side, and above the bar.
pub const MODAL_MARGIN: [u32; 3] = [14, 16, 5];

/// A panel's chrome around its well, top and bottom together: the header, the frame line and
/// the well's shadow line above, and the well's padding and outlines (see panel.rs).
pub const PANEL_CHROME_H: u32 = 23;

/// The page the menus sit in, `[w, h]` in art pixels: the whole screen above the bottom bar.
pub fn page() -> [u32; 2] {
    let px = SPRITE_PX;
    [out_w() / px, out_h() / px - BAR_U]
}

/// Where a modal goes, `[x, y, w, h]` in art pixels from the screen's top left.
pub fn modal_rect() -> [u32; 4] {
    let [w, h] = page();
    let [top, side, bottom] = MODAL_MARGIN;
    [side, top, w - 2 * side, h - top - bottom]
}

/// Where a lone panel `w` by `h` goes: in the middle of the page, rounded down to a whole art
/// pixel as flow.html's `down()` does.
pub fn centred(w: u32, h: u32) -> [u32; 4] {
    let [pw, ph] = page();
    [pw.saturating_sub(w) / 2, ph.saturating_sub(h) / 2, w, h]
}

/// How tall a panel is to hold `content` art pixels in its well.
pub fn panel_height(content: u32) -> u32 {
    content + PANEL_CHROME_H
}

/// An art-pixel position as the screen pixels a `Draw` takes.
pub fn screen_px(u: u32) -> f32 {
    (u * SPRITE_PX) as f32
}
