//! The Nintendo DS Game Card, NTR-005: 33 by 35 mm and 3.8 mm thick, stood upright with its
//! contacts at the foot, the way it goes into the console.
//!
//! It is drawn 8 units to the millimetre, 264 by 280, so the band its sides take their colour
//! from, a few units in from its edge, is the plastic and not the label. Its corners are rounded 1.5 mm; its label
//! fills a shallow well over nearly all the face, as a real card's does, so a scanned label,
//! which carries the NINTENDO DS logo itself, is shown whole; under it a grip groove runs across
//! the foot and an arrow moulded below it points into the slot. The groove, the arrow and the
//! well are relief in the face, not geometry.

use std::sync::OnceLock;

use crate::cart::{Board, Spec, PAD};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_svg, Detail};

const SVG: &str = include_str!("../../assets/nds_card.svg");
const DETAIL_SVG: &str = include_str!("../../assets/nds_card_detail.svg");

/// How thick the card is, as a share of its width: 3.8 mm on its 33.
pub const THICK: f32 = 3.8 / 33.0;

/// How big the card stands on the shelf, as a share of its own size: its face is made at the
/// size it stands at, so its label lands a texel to a pixel there.
pub const SHELF_SCALE: f32 = 1.0;

/// The card's outline, right half, in drawing units (264 by 280), clockwise from the top: each
/// round corner in three cuts. The left half is its mirror.
const HALF: &[(f32, f32)] = &[
    (252.0, 0.0),
    (258.0, 1.6),
    (262.4, 6.0),
    (264.0, 12.0),
    (264.0, 268.0),
    (262.4, 274.0),
    (258.0, 278.4),
    (252.0, 280.0),
];
const UNITS: (f32, f32) = (264.0, 280.0);

/// The size of the face in pixels: about 7 to the millimetre, so the small card's label reads
/// and it stands as tall as a HuCard's label beside it. Its half is an even number of pixels
/// across, so it stands on whole art pixels in the middle of the row.
pub const W: u32 = 232;
pub const H: u32 = 246;

/// How far apart cards stand on the shelf.
pub const PITCH: f32 = 252.0;

/// The label, in the face's pixels: the well's inside, 3.3 mm in at the sides and 1.4 mm down
/// from the top to 30.7 mm, its corners a little round: the shape of ScreenScraper's scans of DS
/// labels, 0.9 as wide as tall, so none of a scan's NINTENDO DS logo or edge is cut.
const LABEL: (u32, u32, u32, u32) = (23, 10, 186, 206);
const LABEL_ROUND: u32 = 3;

/// How far in from the outline a clear card's plastic lightens at its rim, in pixels.
const RIM: u32 = 4;

/// The card's outline in drawing units, all the way round clockwise, and the drawing's size.
pub(crate) fn outline() -> (Vec<(f32, f32)>, (f32, f32)) {
    let mut all = HALF.to_vec();
    all.extend(HALF.iter().rev().map(|&(x, y)| (UNITS.0 - x, y)));
    (all, UNITS)
}

/// The card's silhouette `w` by `h`, as coverage: opaque inside the outline.
pub fn silhouette(w: u32, h: u32) -> Vec<u8> {
    rasterise_svg(SVG, w, h).unwrap_or_else(|| vec![255; (w * h) as usize])
}

fn mask() -> &'static [u8] {
    static MASK: OnceLock<Vec<u8>> = OnceLock::new();
    MASK.get_or_init(|| silhouette(W, H))
}

fn depth() -> &'static [u8] {
    static DEPTH: OnceLock<Vec<u8>> = OnceLock::new();
    DEPTH.get_or_init(|| depth_map(mask(), W as usize, H as usize))
}

fn detail() -> &'static Detail {
    static DETAIL: OnceLock<Detail> = OnceLock::new();
    DETAIL.get_or_init(|| rasterise_detail(DETAIL_SVG, W, H).unwrap_or_else(|| Detail::blank(W, H)))
}

/// How the card's face is laid out. A clear card shows its board: a small one filling the
/// shell, its contacts in a row along its foot.
pub(crate) fn spec() -> Spec {
    Spec {
        w: W,
        h: H,
        label: LABEL,
        label_round: LABEL_ROUND,
        mask: mask(),
        depth: depth(),
        detail: detail(),
        max_h: (LABEL.3 - 2 * PAD) as f32,
        rim: RIM,
        board: Board {
            x: (0.12, 0.88),
            top: 0.1,
            pins: (0.14, 0.86),
            contacts_from: 0.93,
            traces_from: 0.86,
        },
    }
}
