//! A Polaroid, the print a folder of pictures stands as on the Pictures shelf: 88 by 107 mm, thin
//! as paper, its picture the folder's first one over a white chin the folder's name is written on.
//!
//! It is drawn 2.7 pixels to the millimetre, 240 by 291. The picture is the face's label, so the
//! first picture of the folder is cut to fit it as a scan is; the name is set on the chin by
//! `cart::polaroid_chin`.

use std::sync::OnceLock;

use crate::cart::{Board, Spec};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_svg, Detail};

const SVG: &str = include_str!("../../assets/polaroid.svg");
const DETAIL_SVG: &str = include_str!("../../assets/polaroid_detail.svg");

/// How thick the print is, as a share of its width: paper, a little more than a real one so its
/// edge shows when it turns.
pub const THICK: f32 = 0.014;

/// How big the print stands on the shelf, as a share of its own size.
pub const SHELF_SCALE: f32 = 1.0;

pub const W: u32 = 240;
pub const H: u32 = 291;

/// The picture, in the face's pixels: the square the film shows, 79 mm of the print's 88.
pub const LABEL: (u32, u32, u32, u32) = (14, 17, 212, 212);

/// The chin under the picture, where the name is written: `x, y, w, h`.
pub const CHIN: (u32, u32, u32, u32) = (16, 235, 208, 50);

const RIM: u32 = 4;
const CORNER: f32 = 4.0;

pub(crate) fn outline() -> (Vec<(f32, f32)>, (f32, f32)) {
    super::rounded(W as f32, H as f32, CORNER)
}

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

pub(crate) fn spec() -> Spec {
    Spec {
        w: W,
        h: H,
        label: LABEL,
        label_round: 0,
        mask: mask(),
        depth: depth(),
        detail: detail(),
        max_h: (LABEL.3 - 20) as f32,
        rim: RIM,
        board: Board {
            x: (2.0, 2.0),
            top: 0.1,
            pins: (0.15, 0.85),
            contacts_from: 0.9,
            traces_from: 0.8,
        },
    }
}
