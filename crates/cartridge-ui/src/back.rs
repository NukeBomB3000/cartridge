//! What a cart's back is moulded with: a shallow recess, grip ridges, screw wells and the small
//! print every shell carries, drawn as flat shaded shapes laid on the back's own plastic, the way
//! the front's relief is. Each is described as the eye sees the back, left to right, as shares of
//! the cart's drawing; a cart is turned to show its back, so its left is the front's right.

use cartridge_gfx::MeshVertex;

use crate::spin::CartModel;

/// A shape stands this far (as a share of the cart's width) out of the back, a layer to a step,
/// so it draws over what is under it.
const LAYER: f32 = 0.0025;

/// How much light the parts of a back get against the plain plastic, which has `BACK_SHADE`.
const RECESS: f32 = 0.78;
const RIDGE: f32 = 0.7;
const PRINT: f32 = 0.5;
const WELL: f32 = 0.3;
const RIM: f32 = 0.92;

enum Shape {
    /// `x0, y0, x1, y1`, a share of the drawing each.
    Rect([f32; 4], f32, u8),
    /// A round well: middle `x, y`, radius as a share of the width.
    Round([f32; 2], f32, f32, u8),
}

use Shape::{Rect, Round};

/// The lines of small print, as bars in `rows` rows from `y`, each `step` below the last, the
/// widths of lines of type.
fn print(x: f32, y: f32, step: f32, widths: &[f32]) -> Vec<Shape> {
    widths
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let top = y + step * i as f32;
            Rect([x, top, x + w, top + step * 0.45], PRINT, 2)
        })
        .collect()
}

/// A run of grip ridges: `n` lines across `x0..x1`, from `y`, each `step` below the last.
fn ridges(x0: f32, x1: f32, y: f32, step: f32, n: usize) -> Vec<Shape> {
    (0..n)
        .map(|i| {
            let top = y + step * i as f32;
            Rect([x0, top, x1, top + step * 0.4], RIDGE, 1)
        })
        .collect()
}

/// A run of upright ridges down the sides: `n` lines from `x`, each `step` across.
fn grooves(x: f32, y0: f32, y1: f32, step: f32, n: usize) -> Vec<Shape> {
    (0..n)
        .map(|i| {
            let left = x + step * i as f32;
            Rect([left, y0, left + step * 0.4, y1], RIDGE, 1)
        })
        .collect()
}

/// A screw well with its raised rim.
fn screw(x: f32, y: f32, r: f32) -> Vec<Shape> {
    vec![Round([x, y], r * 1.45, RIM, 1), Round([x, y], r, WELL, 2)]
}

fn shapes(model: CartModel) -> Vec<Shape> {
    let mut all = Vec::new();
    match model {
        CartModel::Gba => {
            all.extend(ridges(0.1, 0.9, 0.05, 0.05, 4));
            all.push(Rect([0.14, 0.34, 0.86, 0.9], RECESS, 1));
            all.extend(print(0.2, 0.44, 0.075, &[0.5, 0.38, 0.44]));
            all.extend((0..4).map(|i| {
                let y = 0.7 + 0.04 * i as f32;
                Rect([0.2, y, 0.25, y + 0.025], PRINT, 2)
            }));
        }
        CartModel::Gb(_) => {
            all.extend(grooves(0.1, 0.16, 0.58, 0.035, 3));
            all.extend(grooves(0.79, 0.16, 0.58, 0.035, 3));
            all.push(Rect([0.2, 0.13, 0.8, 0.66], RECESS, 1));
            all.extend(print(0.28, 0.22, 0.07, &[0.44, 0.3, 0.38]));
            all.extend(screw(0.5, 0.86, 0.035));
        }
        CartModel::Snes(_) => {
            all.push(Rect([0.1, 0.25, 0.9, 0.75], RECESS, 1));
            all.extend(print(0.17, 0.35, 0.075, &[0.5, 0.36, 0.44]));
            all.extend(grooves(0.04, 0.18, 0.82, 0.03, 2));
            all.extend(grooves(0.9, 0.18, 0.82, 0.03, 2));
            all.extend(screw(0.07, 0.9, 0.025));
            all.extend(screw(0.93, 0.9, 0.025));
        }
        CartModel::Nes(_) => {
            all.extend(ridges(0.12, 0.88, 0.06, 0.045, 4));
            all.push(Rect([0.12, 0.3, 0.88, 0.8], RECESS, 1));
            all.extend(print(0.19, 0.4, 0.075, &[0.46, 0.34, 0.4]));
            all.extend(screw(0.18, 0.9, 0.03));
            all.extend(screw(0.82, 0.9, 0.03));
        }
        CartModel::Nds => {
            all.push(Rect([0.14, 0.1, 0.86, 0.74], RECESS, 1));
            all.extend(print(0.2, 0.2, 0.08, &[0.5, 0.3, 0.42]));
            all.extend(ridges(0.2, 0.8, 0.84, 0.04, 3));
        }
        CartModel::HuCard | CartModel::Md(_) | CartModel::Sega8(_) => {
            all.push(Rect([0.12, 0.2, 0.88, 0.7], RECESS, 1));
            all.extend(print(0.19, 0.3, 0.08, &[0.5, 0.34, 0.42]));
            all.extend(ridges(0.12, 0.88, 0.8, 0.04, 3));
        }
        CartModel::Disc => {}
    }
    all
}

/// The shapes moulded on the back of `model`, `thick` behind its face, taking the colour of the
/// plastic at `uv`: `shade` is the plain back's.
pub(crate) fn relief(
    model: CartModel,
    uv: [f32; 2],
    (uw, uh): (f32, f32),
    thick: f32,
    shade: f32,
) -> Vec<MeshVertex> {
    let vertex = |x: f32, y: f32, layer: u8, k: f32| MeshVertex {
        // seen from behind, the left is the front's right
        xyz: [1.0 - x, y, thick + LAYER * layer as f32],
        normal: [0.0, 0.0, 1.0],
        uv,
        shade: shade * k,
    };
    // a drawing is not square: a round well is round in the drawing's own units
    let aspect = uw / uh;
    let mut out = Vec::new();
    for shape in shapes(model) {
        match shape {
            Rect([x0, y0, x1, y1], k, layer) => {
                let (a, b, c, d) = (
                    vertex(x0, y0, layer, k),
                    vertex(x1, y0, layer, k),
                    vertex(x1, y1, layer, k),
                    vertex(x0, y1, layer, k),
                );
                out.extend([a, b, c, a, c, d]);
            }
            Round([x, y], r, k, layer) => {
                const CUTS: usize = 10;
                let at = |i: usize| {
                    let t = i as f32 / CUTS as f32 * std::f32::consts::TAU;
                    vertex(x + r * t.cos(), y + r * aspect * t.sin(), layer, k)
                };
                let middle = vertex(x, y, layer, k);
                for i in 0..CUTS {
                    out.extend([middle, at(i), at(i + 1)]);
                }
            }
        }
    }
    out
}
