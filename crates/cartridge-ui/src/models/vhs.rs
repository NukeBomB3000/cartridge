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

/// The reels' centres, in drawing units, and the radii a roll of tape is between: round the
/// bare hub, and as thick as the window holds.
const REELS: [(f32, f32); 2] = [(55.0, 105.0), (318.0, 105.0)];
const HUB_R: f32 = 17.0;
const FULL_R: f32 = 40.0;

/// How much of the tape is on each of the two reels, `(left, right)`, as shares of a full one,
/// for the tape named `title`: always the same for a name, differing from name to name, and
/// what the one has lost the other has taken, as on a real cassette.
pub(crate) fn fills(title: &str) -> (f32, f32) {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in title.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let left = 0.1 + 0.8 * ((h >> 24) % 1000) as f32 / 999.0;
    (left, 1.0 - left)
}

/// The radius of a roll holding `fill` of a full one: tape is wound by area.
pub(crate) fn roll_radius(fill: f32) -> f32 {
    (HUB_R * HUB_R + fill.clamp(0.0, 1.0) * (FULL_R * FULL_R - HUB_R * HUB_R)).sqrt()
}

/// Paints the tape rolls on the two reels, and cuts away what of each roll's disc in the face the
/// tape does not fill, so the window shows through round it.
fn roll(face: &mut CartFace, title: &str) {
    const TAPE: [u8; 3] = [0x2a, 0x1f, 0x18];
    let (l, r) = fills(title);
    for ((cx, cy), fill) in REELS.into_iter().zip([l, r]) {
        let radius = roll_radius(fill);
        let (x0, x1) = ((cx - FULL_R - 2.0) as i32, (cx + FULL_R + 2.0) as i32);
        let (y0, y1) = ((cy - FULL_R - 2.0) as i32, (cy + FULL_R + 2.0) as i32);
        for y in y0.max(0)..y1.min(face.h as i32) {
            for x in x0.max(0)..x1.min(face.w as i32) {
                let i = y as usize * face.w as usize + x as usize;
                // only the discs standing in a window are the roll's to cut: the shell's own pixels
                // under them are left
                let in_window = WINDOWS.iter().any(|&(wx, wy, ww, wh, _)| {
                    (x as f32) > wx
                        && (x as f32) < wx + ww
                        && (y as f32) > wy
                        && (y as f32) < wy + wh
                });
                if !in_window || mask()[i] == 0 {
                    continue;
                }
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let p = i * 4;
                if d > radius {
                    face.rgba[p + 3] = 0;
                    continue;
                }
                // the tape: dark, a hair lighter towards the rim, in faint rings
                let rim = ((d - HUB_R) / (radius - HUB_R).max(1.0)).clamp(0.0, 1.0);
                let ring = if (d as i32) % 4 == 0 { 0.9 } else { 1.0 };
                let edge = if radius - d < 1.5 {
                    1.9
                } else {
                    1.0 + 0.35 * rim
                };
                for c in 0..3 {
                    face.rgba[p + c] = (TAPE[c] as f32 * edge * ring).min(255.0) as u8;
                }
                face.rgba[p + 3] = 255;
            }
        }
    }
}

/// Paints the lid's lettering, the reels, the foot of the label and the VHS mark over `face`,
/// and the tape rolls on the reels, filled as the tape named `title` has them.
pub(crate) fn paint(face: &mut CartFace, title: &str) {
    roll(face, title);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_reels_share_the_tape_and_a_name_always_has_the_same_fill() {
        for title in ["Twin Peaks", "Leg and Glute", "Film", ""] {
            let (l, r) = fills(title);
            assert!((l + r - 1.0).abs() < 1e-5, "{title}: {l} {r}");
            assert!((0.1..=0.9).contains(&l), "{title}: {l}");
            assert_eq!(fills(title), (l, r));
        }
        assert_ne!(fills("Twin Peaks"), fills("Leg and Glute"));
    }

    #[test]
    fn a_roll_is_as_thick_as_its_fill_and_never_beyond_the_window() {
        assert!((roll_radius(0.0) - HUB_R).abs() < 1e-4);
        assert!((roll_radius(1.0) - FULL_R).abs() < 1e-4);
        assert!(roll_radius(0.3) < roll_radius(0.7));
    }

    #[test]
    fn the_window_shows_through_round_a_thin_roll_and_not_through_a_thick_one() {
        let s = spec();
        let mut thin = CartFace {
            rgba: vec![255; (W * H * 4) as usize],
            w: W,
            h: H,
        };
        let mut thick = CartFace {
            rgba: thin.rgba.clone(),
            w: W,
            h: H,
        };
        // a name's two reels differ: compare the sides of one face
        roll(&mut thin, "Twin Peaks");
        let (l, r) = fills("Twin Peaks");
        let (thin_side, thick_side) = if l < r { (0usize, 1usize) } else { (1, 0) };
        let cut = |face: &CartFace, side: usize| {
            let (cx, cy) = REELS[side];
            // a pixel of the window 30 px from the hub, which a thin roll leaves clear
            let x = (cx + 30.0) as usize;
            let y = cy as usize;
            let i = y * W as usize + x;
            (s.mask[i] > 0, face.rgba[i * 4 + 3])
        };
        let _ = &mut thick;
        let (inside, alpha_thin) = cut(&thin, thin_side);
        assert!(inside, "the disc stands in the mask there");
        let radius_thin = roll_radius(l.min(r));
        if radius_thin < 30.0 {
            assert_eq!(
                alpha_thin, 0,
                "a thin roll leaves the window clear 30 px out"
            );
        }
        let radius_thick = roll_radius(l.max(r));
        let (_, alpha_thick) = cut(&thin, thick_side);
        if radius_thick > 31.0 {
            assert_eq!(alpha_thick, 255, "a thick roll fills it");
        }
    }
}
