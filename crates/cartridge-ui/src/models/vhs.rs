//! A VHS cassette, the tape a folder of videos stands as on the Video shelf: 188 by 104 mm and
//! 25 mm thick, lying on its long edge, its lid across the top and a window for each reel to
//! either side of the label.
//!
//! It is drawn two pixels to the millimetre, 368 by 196. The outline's lid stands in from the
//! body by a pixel or two; each window is cut right through the shell (so is the back), a reel
//! standing in it as a disc in the face, and the lid's and the foot's ridges are relief.

use std::sync::OnceLock;

use crate::cart::{Board, CartFace, Spec};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_rgba, rasterise_svg, Detail};

const SVG: &str = include_str!("../../assets/vhs.svg");
const DETAIL_SVG: &str = include_str!("../../assets/vhs_detail.svg");
const PAINT_SVG: &str = include_str!("../../assets/vhs_paint.svg");

/// How thick the cassette is, as a share of its width: 25.4 mm on its 188.
pub const THICK: f32 = 25.4 / 188.0;

/// How big the cassette stands on the shelf, as a share of its own size.
pub const SHELF_SCALE: f32 = 1.0;

pub const W: u32 = 368;
pub const H: u32 = 196;

/// The label: the black panel between the windows, above the strip along its foot that is
/// painted. The tape's name is set in gold on it.
const LABEL: (u32, u32, u32, u32) = (107, 60, 155, 66);

/// How far in from the outline the plastic lightens at its rim, in pixels.
const RIM: u32 = 5;

/// The cassette's outline in drawing units, all the way round clockwise from the top: the lid
/// a little narrower than the body below it, the corners cut.
const OUTLINE: &[(f32, f32)] = &[
    (7.0, 0.0),
    (361.0, 0.0),
    (364.5, 1.5),
    (366.0, 5.0),
    (366.0, 50.0),
    (368.0, 53.0),
    (368.0, 189.0),
    (367.0, 193.0),
    (364.0, 195.5),
    (361.0, 196.0),
    (7.0, 196.0),
    (4.0, 195.5),
    (1.0, 193.0),
    (0.0, 189.0),
    (0.0, 53.0),
    (3.0, 50.0),
    (3.0, 5.0),
    (4.5, 1.5),
];

pub(crate) fn outline() -> (Vec<(f32, f32)>, (f32, f32)) {
    (OUTLINE.to_vec(), (W as f32, H as f32))
}

/// The reel windows, `x, y, width, height` and the radius of their corners, in drawing units.
const WINDOWS: [(f32, f32, f32, f32, f32); 2] = [
    (11.0, 60.0, 83.0, 90.0, 16.0),
    (275.0, 59.0, 83.0, 91.0, 16.0),
];

/// The windows' contours, each wound the other way round from the outline so their walls face
/// into the window.
pub(crate) fn holes() -> Vec<Vec<(f32, f32)>> {
    const CUTS: usize = 6;
    WINDOWS
        .into_iter()
        .map(|(x, y, w, h, r)| {
            // clockwise from the top left corner's end, each corner in `CUTS` cuts
            let corners = [
                (x + r, y + r, 180.0f32),
                (x + w - r, y + r, 270.0),
                (x + w - r, y + h - r, 0.0),
                (x + r, y + h - r, 90.0),
            ];
            let mut points = Vec::new();
            for (cx, cy, from) in corners {
                for k in 0..=CUTS {
                    let a = (from + 90.0 * k as f32 / CUTS as f32).to_radians();
                    points.push((cx + r * a.cos(), cy + r * a.sin()));
                }
            }
            points.reverse();
            points
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
        max_h: (LABEL.3 - 16) as f32,
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

/// Paints the lid's lettering, the reels, the foot of the label and the VHS mark over `face`.
pub(crate) fn paint(face: &mut CartFace) {
    static PAINT: OnceLock<Vec<u8>> = OnceLock::new();
    let over = PAINT.get_or_init(|| rasterise_rgba(PAINT_SVG, W, H).unwrap_or_default());
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
