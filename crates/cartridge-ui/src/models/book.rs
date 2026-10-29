//! A hardback book, what an EPUB stands as on the Books shelf: 130 by 200 mm and 28 mm thick,
//! its spine on the left, its cover the EPUB's own picture.
//!
//! It is drawn 1.46 pixels to the millimetre, 190 by 292. The face is the cover, with a band down
//! the left for the spine and a thin cream edge along the other three sides: a side of the model
//! is coloured from the face just inside its edge, so those bands are what the spine and the
//! page block are coloured from.

use std::sync::OnceLock;

use cartridge_gfx::MeshVertex;

use crate::cart::{Board, CartFace, Spec};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_svg, Detail};

const SVG: &str = include_str!("../../assets/book.svg");
const DETAIL_SVG: &str = include_str!("../../assets/book_detail.svg");

/// How thick the book is, as a share of its width: 28 mm on its 130.
pub const THICK: f32 = 28.0 / 130.0;

/// How big the book stands on the shelf, as a share of its own size.
pub const SHELF_SCALE: f32 = 1.0;

/// How far apart books stand on the shelf: the width, the round of the spine that stands out
/// beyond it on the left (half the thickness), and a gap, so neighbours turned to the eye do not
/// run into each other.
pub const PITCH: f32 = W as f32 + (THICK * W as f32) / 2.0 + 34.0;

pub const W: u32 = 190;
pub const H: u32 = 292;

/// How wide the spine's band is down the left of the face, and the cream page edge along the
/// other three sides, in pixels.
pub const SPINE: u32 = 6;
pub const EDGE: u32 = 2;

/// The cover, in the face's pixels: all the face inside those bands.
pub const LABEL: (u32, u32, u32, u32) = (SPINE, EDGE, W - SPINE - EDGE, H - 2 * EDGE);

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
        max_h: (LABEL.3 - 24) as f32,
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

/// Paints the spine's band and the page edge round the cover on `face`: the spine the cloth's
/// colour, darker, the pages cream.
pub(crate) fn paint(face: &mut CartFace, cloth: [u8; 3]) {
    const PAGES: [u8; 3] = [0xe9, 0xdf, 0xc4];
    let spine = cloth.map(|c| (c as f32 * 0.6) as u8);
    for y in 0..face.h {
        for x in 0..face.w {
            let band = if x < SPINE {
                Some(spine)
            } else if x >= face.w - EDGE || y < EDGE || y >= face.h - EDGE {
                Some(PAGES)
            } else {
                None
            };
            if let Some(c) = band {
                let i = ((y * face.w + x) * 4) as usize;
                face.rgba[i..i + 3].copy_from_slice(&c);
            }
        }
    }
}

/// How thick a cover board is, and how far the boards stand out past the page block along the
/// right, top and foot, as shares of the book's width (z, x) and height (y): 3 mm each.
const BOARD: f32 = 3.0 / 130.0;
const OVERHANG_X: f32 = 3.0 / 130.0;
const OVERHANG_Y: f32 = 3.0 / 200.0;
/// How many lines the page block's sides are striped with, and the share of light the dark ones
/// take: a stack of leaves, seen edge on.
const LEAVES: usize = 22;
const LEAF_SHADE: f32 = 0.84;
/// How many cuts the round of the spine is made in.
const ROUND_CUTS: usize = 14;

/// The book in few polygons: the cover face (the cover's own picture), a rounded spine, the two
/// boards' edges, the page block set back from the boards along the right, top and foot with its
/// leaves lined across its sides, and the back board.
pub(crate) fn mesh() -> Vec<MeshVertex> {
    let thick = THICK;
    let (w, h) = (W as f32, H as f32);
    // where the sides take their colour: the spine's band for cloth, the cream band for paper
    let cloth = [SPINE as f32 * 0.5 / w, 0.5];
    let right = [(w - 1.0) / w, 0.5];
    let top = [0.5, 1.0 / h];
    let foot = [0.5, (h - 1.0) / h];
    let v = |x: f32, y: f32, z: f32, normal: [f32; 3], uv: [f32; 2], shade: f32| MeshVertex {
        xyz: [x, y, z],
        normal,
        uv,
        shade,
    };
    let quad =
        |out: &mut Vec<MeshVertex>, a: MeshVertex, b: MeshVertex, c: MeshVertex, d: MeshVertex| {
            out.extend([a, b, c, a, c, d]);
        };
    let mut out = Vec::new();

    // the spine, a half round from the front board's edge to the back board's, bulging left
    let radius = thick / 2.0;
    for k in 0..ROUND_CUTS {
        let (t0, t1) = (
            std::f32::consts::PI * k as f32 / ROUND_CUTS as f32,
            std::f32::consts::PI * (k + 1) as f32 / ROUND_CUTS as f32,
        );
        let at = |t: f32| (-radius * t.sin(), radius - radius * t.cos());
        // one normal for the whole strip, the middle of its two ends: the shader turns a polygon away
        // by its vertices' normals, so a strip whose corners disagree would be pulled across the
        // screen as it turns edge on; the round is smooth by the light its corners are given
        let tm = (t0 + t1) / 2.0;
        let n = [-tm.sin(), 0.0, -tm.cos()];
        // the middle of the round catches the most light
        let (s0, s1) = (0.55 + 0.45 * t0.sin(), 0.55 + 0.45 * t1.sin());
        let ((x0, z0), (x1, z1)) = (at(t0), at(t1));
        quad(
            &mut out,
            v(x0, 0.0, z0, n, cloth, s0),
            v(x0, 1.0, z0, n, cloth, s0),
            v(x1, 1.0, z1, n, cloth, s1),
            v(x1, 0.0, z1, n, cloth, s1),
        );
    }

    // the boards' edges along the right, top and foot, the front's and the back's
    let edges: [([f32; 3], bool); 3] = [
        ([1.0, 0.0, 0.0], true),
        ([0.0, -1.0, 0.0], false),
        ([0.0, 1.0, 0.0], false),
    ];
    for (z0, z1) in [(0.0, BOARD), (thick - BOARD, thick)] {
        for (n, is_right) in edges {
            let (a, b) = if is_right {
                ((1.0, 0.0), (1.0, 1.0))
            } else if n[1] < 0.0 {
                ((0.0, 0.0), (1.0, 0.0))
            } else {
                ((0.0, 1.0), (1.0, 1.0))
            };
            quad(
                &mut out,
                v(a.0, a.1, z0, n, cloth, 0.9),
                v(b.0, b.1, z0, n, cloth, 0.9),
                v(b.0, b.1, z1, n, cloth, 0.9),
                v(a.0, a.1, z1, n, cloth, 0.9),
            );
        }
    }

    // the page block, set back from the boards, its leaves lined across the sides it shows
    let (xr, yt, yb) = (1.0 - OVERHANG_X, OVERHANG_Y, 1.0 - OVERHANG_Y);
    let (p0, p1) = (BOARD, thick - BOARD);
    for i in 0..LEAVES {
        let z0 = p0 + (p1 - p0) * i as f32 / LEAVES as f32;
        let z1 = p0 + (p1 - p0) * (i + 1) as f32 / LEAVES as f32;
        let shade = if i % 2 == 0 { 1.0 } else { LEAF_SHADE };
        quad(
            &mut out,
            v(xr, yt, z0, [1.0, 0.0, 0.0], right, shade),
            v(xr, yb, z0, [1.0, 0.0, 0.0], right, shade),
            v(xr, yb, z1, [1.0, 0.0, 0.0], right, shade),
            v(xr, yt, z1, [1.0, 0.0, 0.0], right, shade),
        );
        quad(
            &mut out,
            v(0.0, yt, z0, [0.0, -1.0, 0.0], top, shade),
            v(xr, yt, z0, [0.0, -1.0, 0.0], top, shade),
            v(xr, yt, z1, [0.0, -1.0, 0.0], top, shade),
            v(0.0, yt, z1, [0.0, -1.0, 0.0], top, shade),
        );
        quad(
            &mut out,
            v(0.0, yb, z0, [0.0, 1.0, 0.0], foot, shade),
            v(xr, yb, z0, [0.0, 1.0, 0.0], foot, shade),
            v(xr, yb, z1, [0.0, 1.0, 0.0], foot, shade),
            v(0.0, yb, z1, [0.0, 1.0, 0.0], foot, shade),
        );
    }

    // the back board
    quad(
        &mut out,
        v(0.0, 0.0, thick, [0.0, 0.0, 1.0], cloth, 0.8),
        v(1.0, 0.0, thick, [0.0, 0.0, 1.0], cloth, 0.8),
        v(1.0, 1.0, thick, [0.0, 0.0, 1.0], cloth, 0.8),
        v(0.0, 1.0, thick, [0.0, 0.0, 1.0], cloth, 0.8),
    );

    // the cover, the whole texture laid over a few cells as the other models' faces are
    let (cols, rows) = (4usize, 3usize);
    for r in 0..rows {
        for c in 0..cols {
            let at = |c: usize, r: usize| {
                let (u, w) = (c as f32 / cols as f32, r as f32 / rows as f32);
                v(u, w, 0.0, [0.0, 0.0, -1.0], [u, w], 1.0)
            };
            quad(
                &mut out,
                at(c, r),
                at(c + 1, r),
                at(c + 1, r + 1),
                at(c, r + 1),
            );
        }
    }
    out
}

#[cfg(test)]
mod mesh_tests {
    use super::*;

    /// The shader turns a polygon away by its vertices' normals: every polygon of the book has
    /// one normal for all of its corners, or it would be pulled across the screen edge on.
    #[test]
    fn every_polygon_has_one_normal_for_all_its_corners() {
        let m = mesh();
        for tri in m.chunks_exact(3) {
            assert_eq!(tri[0].normal, tri[1].normal);
            assert_eq!(tri[1].normal, tri[2].normal);
        }
    }

    #[test]
    fn the_round_of_the_spine_stands_left_of_the_cover_and_the_pages_are_set_back() {
        let m = mesh();
        let min_x = m.iter().map(|v| v.xyz[0]).fold(f32::MAX, f32::min);
        assert!(
            (min_x + THICK / 2.0).abs() < 1e-5,
            "the spine bulges half the thickness: {min_x}"
        );
        // the page block's right side stands in from the cover's edge
        let pages = m
            .iter()
            .filter(|v| v.normal == [1.0, 0.0, 0.0] && v.xyz[0] < 1.0)
            .count();
        assert!(pages > 0, "the pages' side is inside the cover's edge");
    }
}
