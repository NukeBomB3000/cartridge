//! The PSP's UMD, the Universal Media Disc in its shell: 64 by 63 mm and 4.2 mm thick, stood
//! upright the way it goes into the console. Its top is a half circle the width of the shell,
//! its sides run straight down, and its foot curves a little, its corners rounded over; a big
//! round window across the face shows the disc, whose printed side is the picture on a UMD, so
//! the label is a circle and a scan of one fills it whole. The shell is pale; the window's lip is
//! relief in the face, not geometry.
//!
//! The outline is the real shell's, traced from a photograph of one and drawn 8 units to the
//! millimetre, 512 by 506.

use std::sync::OnceLock;

use crate::cart::{Board, Spec, PAD};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_svg, Detail};

const SVG: &str = include_str!("../../assets/umd.svg");
const DETAIL_SVG: &str = include_str!("../../assets/umd_detail.svg");

/// How thick the shell is, as a share of its width: 4.2 mm on its 64.
pub const THICK: f32 = 4.2 / 64.0;

/// How big the shell stands on the shelf, as a share of its own size: its face is made at the
/// size it stands at, so its label lands a texel to a pixel there.
pub const SHELF_SCALE: f32 = 1.0;

/// The shell's outline, right half, in drawing units, clockwise from the top. The left half is
/// its mirror.
const HALF: &[(f32, f32)] = &[
    (256.0, 0.0),
    (288.0, 2.2),
    (319.5, 8.2),
    (348.9, 17.4),
    (373.0, 28.2),
    (400.2, 45.7),
    (428.5, 69.3),
    (448.5, 90.5),
    (467.7, 116.8),
    (480.4, 140.0),
    (492.2, 167.1),
    (499.0, 190.7),
    (503.2, 211.5),
    (507.8, 265.0),
    (512.0, 407.2),
    (511.6, 419.9),
    (510.2, 426.3),
    (505.8, 435.9),
    (501.0, 441.9),
    (495.8, 446.3),
    (484.0, 453.5),
    (447.9, 469.5),
    (423.3, 478.3),
    (394.2, 487.0),
    (353.4, 496.2),
    (309.3, 502.6),
    (256.0, 505.6),
];
const UNITS: (f32, f32) = (512.0, 505.6);

/// The size of the face in pixels: about 3.6 to the millimetre, so its label reads and the
/// shell stands about as tall as a DS card beside it. Its half is an even number of pixels
/// across, so it stands on whole art pixels in the middle of the row.
pub const W: u32 = 232;
pub const H: u32 = 230;

/// How far apart shells stand on the shelf.
pub const PITCH: f32 = 252.0;

/// The label, in the face's pixels: the window's inside, a circle 56 mm across, 3 mm down from
/// the top of the shell and as far from either side: the shape of ScreenScraper's scans of
/// UMDs, so none of a scan's rim is cut.
const LABEL: (u32, u32, u32, u32) = (15, 11, 202, 202);
const LABEL_ROUND: u32 = 101;

/// How far in from the outline a clear shell's plastic lightens at its rim, in pixels.
const RIM: u32 = 4;

/// The shell's outline in drawing units, all the way round clockwise, and the drawing's size.
pub(crate) fn outline() -> (Vec<(f32, f32)>, (f32, f32)) {
    let mut all = HALF.to_vec();
    all.extend(
        HALF[1..HALF.len() - 1]
            .iter()
            .rev()
            .map(|&(x, y)| (UNITS.0 - x, y)),
    );
    (all, UNITS)
}

/// The shell's silhouette `w` by `h`, as coverage: opaque inside the outline.
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

/// How the shell's face is laid out. A clear shell shows the disc's board: a small one filling
/// it, its contacts in a row along its foot.
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
