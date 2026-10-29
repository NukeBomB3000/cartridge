//! A compact cassette, the tape a folder of music stands as on the Music shelf: 100.4 by 63.8 mm
//! and 12 mm thick, lying on its long edge as it goes into a deck, its label on top and its
//! window, with the two spools in it, under the label.
//!
//! It is drawn 3.35 pixels to the millimetre, 336 by 214. The label is the paper strip along the
//! top, where the folder's name is printed; the window, the spools, the head's trapezoid and the
//! screws are painted over the plastic from a coloured drawing, as they are not a lit or a
//! shaded relief.

use std::sync::OnceLock;

use crate::cart::{Board, CartFace, Spec, PAD};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_rgba, rasterise_svg, Detail};

const SVG: &str = include_str!("../../assets/tape.svg");
const DETAIL_SVG: &str = include_str!("../../assets/tape_detail.svg");
const PAINT_SVG: &str = include_str!("../../assets/tape_paint.svg");

/// How thick the cassette is, as a share of its width: 12 mm on its 100.4.
pub const THICK: f32 = 12.0 / 100.4;

/// How big the cassette stands on the shelf, as a share of its own size.
pub const SHELF_SCALE: f32 = 1.0;

pub const W: u32 = 336;
pub const H: u32 = 214;

/// The label, in the face's pixels: the white strip across the top of the paper, where a
/// cassette's name is written; the coloured band under it is painted.
const LABEL: (u32, u32, u32, u32) = (23, 14, 286, 44);

/// How far in from the outline the plastic lightens at its rim, in pixels.
const RIM: u32 = 5;

/// The shell's corners are rounded 3 mm, in drawing units, each cut in four.
const CORNER: f32 = 8.0;

/// The cassette's outline in drawing units, all the way round clockwise from the top left, and
/// the drawing's size.
pub(crate) fn outline() -> (Vec<(f32, f32)>, (f32, f32)) {
    let (w, h) = (W as f32, H as f32);
    let mut points = Vec::new();
    // each corner as its centre and the angle it starts at, going clockwise
    let corners = [
        (CORNER, CORNER, 180.0f32),
        (w - CORNER, CORNER, 270.0),
        (w - CORNER, h - CORNER, 0.0),
        (CORNER, h - CORNER, 90.0),
    ];
    for (cx, cy, from) in corners {
        for step in 0..=3 {
            let a = (from + 30.0 * step as f32).to_radians();
            points.push((cx + CORNER * a.cos(), cy + CORNER * a.sin()));
        }
    }
    (points, (w, h))
}

/// The holes through the shell, as the drawing has them: each hub's, and the four in the head's
/// trapezoid at the foot. Centre `x, y` and radius, in drawing units.
const HOLES: [(f32, f32, f32); 6] = [
    (96.0, 106.0, 13.5),
    (236.0, 106.0, 13.5),
    (85.0, 196.0, 5.5),
    (243.0, 196.0, 5.5),
    (119.0, 196.0, 4.0),
    (210.0, 196.0, 4.0),
];

/// The holes' contours, each wound the other way round from the outline so their walls face
/// into the hole.
pub(crate) fn holes() -> Vec<Vec<(f32, f32)>> {
    const CUTS: usize = 24;
    HOLES
        .into_iter()
        .map(|(cx, cy, r)| {
            (0..CUTS)
                .map(|i| {
                    let a = -(i as f32) * std::f32::consts::TAU / CUTS as f32;
                    (cx + r * a.cos(), cy + r * a.sin())
                })
                .collect()
        })
        .collect()
}

/// The cassette's silhouette `w` by `h`, as coverage.
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

/// How the cassette's face is laid out.
pub(crate) fn spec() -> Spec {
    Spec {
        w: W,
        h: H,
        label: LABEL,
        label_round: 0,
        mask: mask(),
        depth: depth(),
        detail: detail(),
        max_h: (LABEL.3 - 2 * PAD) as f32,
        rim: RIM,
        board: Board {
            // an empty range: a clear shell has no board to show
            x: (2.0, 2.0),
            top: 0.1,
            pins: (0.15, 0.85),
            contacts_from: 0.9,
            traces_from: 0.8,
        },
    }
}

/// Paints the paper's colour band with its window, the hubs, the black bar, the rollers and the
/// screws over `face`; `band` is the paper's colour.
pub(crate) fn paint(face: &mut CartFace, band: [u8; 3]) {
    let hex = |c: [u8; 3]| format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]);
    let svg = PAINT_SVG.replace("@BAND@", &hex(band));
    let over = rasterise_rgba(&svg, W, H).unwrap_or_default();
    for (px, top) in face.rgba.chunks_exact_mut(4).zip(over.chunks_exact(4)) {
        let a = top[3] as u32;
        if a == 0 {
            continue;
        }
        for c in 0..3 {
            px[c] = ((top[c] as u32 * a + px[c] as u32 * (255 - a) + 127) / 255) as u8;
        }
        px[3] = px[3].max(top[3]);
    }
}
