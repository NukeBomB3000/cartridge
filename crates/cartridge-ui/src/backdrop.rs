use std::path::Path;

use cartridge_gfx::{out_h, out_w, Draw, TexId};
use cartridge_store::{BackdropStyle, BACKDROP_COLOURS};

use crate::art;
use crate::plate::UndoFace;

const SCRIM: f32 = 0.62;

pub fn wallpaper_face(path: &Path) -> Option<Vec<u8>> {
    art::cover(path, out_w(), out_h())
}

pub fn draw_backdrop(face: Option<TexId>, out: &mut Vec<Draw>) {
    let Some(tex) = face else {
        return;
    };
    out.push(Draw::Tex {
        x: 0.0,
        y: 0.0,
        w: out_w() as f32,
        h: out_h() as f32,
        tex,
        alpha: 1.0,
    });
    out.push(Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: out_w() as f32,
        h: out_h() as f32,
        colour: [0.0, 0.0, 0.0, SCRIM],
    });
}

/// What each of [`BACKDROP_COLOURS`] looks like: dark enough that a cart's label always
/// reads over it, the first the near-black cartridge drew before it had a choice.
pub const BACKDROP_PALETTE: [[u8; 3]; BACKDROP_COLOURS.len()] = [
    [0x05, 0x05, 0x08],
    [0x00, 0x00, 0x00],
    [0x22, 0x25, 0x2c],
    [0x30, 0x0f, 0x16],
    [0x3a, 0x18, 0x10],
    [0x2a, 0x1c, 0x12],
    [0x30, 0x22, 0x08],
    [0x26, 0x2a, 0x0c],
    [0x10, 0x2c, 0x16],
    [0x0e, 0x2e, 0x26],
    [0x0b, 0x2b, 0x2e],
    [0x0c, 0x2a, 0x3e],
    [0x10, 0x1b, 0x36],
    [0x1a, 0x16, 0x40],
    [0x29, 0x14, 0x31],
    [0x30, 0x0c, 0x22],
    [0x36, 0x12, 0x2a],
];

/// A colour's name to show, from the key cartridge.state keeps it by.
pub fn backdrop_colour_name(colour: u8) -> &'static str {
    match BACKDROP_COLOURS.get(colour as usize).copied() {
        Some("black") => "Black",
        Some("slate") => "Slate",
        Some("navy") => "Navy",
        Some("teal") => "Teal",
        Some("green") => "Green",
        Some("plum") => "Plum",
        Some("crimson") => "Crimson",
        Some("amber") => "Amber",
        Some("indigo") => "Indigo",
        Some("sky") => "Sky",
        Some("mint") => "Mint",
        Some("olive") => "Olive",
        Some("rose") => "Rose",
        Some("coral") => "Coral",
        Some("brown") => "Brown",
        Some("wine") => "Wine",
        _ => "Night",
    }
}

pub fn backdrop_rgb(colour: u8) -> [u8; 3] {
    BACKDROP_PALETTE[(colour as usize).min(BACKDROP_PALETTE.len() - 1)]
}

/// How far a pattern goes before it looks the same again: a tiled pattern's square, the bits'
/// scatter.
/// Both divide 640, 720 and 480, so a still pattern sits square on every canvas.
/// The PSP's waves are one picture the canvas wide, so they come round again only once the
/// canvas has gone by.
pub fn backdrop_period(style: BackdropStyle) -> u32 {
    match style {
        BackdropStyle::Bits | BackdropStyle::Switch => 80,
        BackdropStyle::Psp => out_w(),
        _ => 40,
    }
}

/// How fast a drifting pattern moves, in canvas pixels a second: slow enough to sit behind the
/// shelf rather than pull the eye from it, and a whole number of milliseconds to a period.
pub fn backdrop_speed(style: BackdropStyle) -> f64 {
    match style {
        BackdropStyle::Bits => 10.0,
        _ => 16.0,
    }
}

/// How far left a pattern has drifted `ms` milliseconds in, in whole art pixels (`ART` screen
/// pixels at a time) so its pixels stay on the grid the rest of the UI is drawn on, starting
/// over each period.
pub fn backdrop_shift(style: BackdropStyle, moving: bool, ms: u64) -> u32 {
    if !moving || !style.moves() {
        return 0;
    }
    let moved = ms as f64 / 1000.0 * backdrop_speed(style);
    let art = (moved / ART as f64).floor() as u64 * ART as u64;
    (art % backdrop_period(style) as u64) as u32
}

/// `base` lifted toward white by `lift`, or sunk toward black by as much when it is below 0.
fn tint(base: [u8; 3], lift: f32) -> [u8; 4] {
    let c = |v: u8| {
        let v = v as f32;
        let to = if lift < 0.0 {
            v * (1.0 + lift)
        } else {
            v + (255.0 - v) * lift
        };
        to.round() as u8
    };
    [c(base[0]), c(base[1]), c(base[2]), 255]
}

/// One of a tiled pattern's pixels is 2x2 of the canvas's, as the words' and the sprites' are.
const ART: u32 = 2;
/// A tiled pattern's square, in its own pixels: one period.
const TILE: u32 = 20;
const BIT: u32 = 4;

/// Ordered dither thresholds, out of 16.
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// A plate lit from the top left: a light edge above and to the left, a dark one below and to
/// the right, and a face a shade above the colour so the dark edge shows even on Night.
fn plate(tx: u32, ty: u32) -> f32 {
    let (top, left) = (ty == 0, tx == 0);
    let (bottom, right) = (ty == TILE - 1, tx == TILE - 1);
    if (top && right) || (bottom && left) {
        0.045
    } else if top || left {
        0.13
    } else if bottom || right {
        -0.6
    } else {
        0.045
    }
}

/// How far the pixel (`tx`, `ty`) of a tiled pattern's square lifts the colour (see [`tint`]).
fn tile_lift(style: BackdropStyle, tx: u32, ty: u32) -> f32 {
    let last = TILE - 1;
    match style {
        BackdropStyle::Plates => plate(tx, ty),
        BackdropStyle::Rivets => {
            let at = |v: u32| v == 2 || v == last - 2;
            if at(tx) && at(ty) {
                0.2
            } else if tx > 0 && ty > 0 && at(tx - 1) && at(ty - 1) {
                -0.6
            } else {
                plate(tx, ty)
            }
        }
        BackdropStyle::Dotted => {
            let cross =
                (tx == 0 && [0, 1, last].contains(&ty)) || (ty == 0 && [1, last].contains(&tx));
            if cross {
                0.17
            } else if (tx == 0 && ty % 2 == 1) || (ty == 0 && tx % 2 == 1) {
                0.09
            } else {
                0.0
            }
        }
        BackdropStyle::Dither => {
            if tx == 0 || ty == 0 {
                return 0.1;
            }
            let light = 1.0 - (tx + ty - 2) as f32 / (2 * (TILE - 2)) as f32;
            if (BAYER[(ty % 4) as usize][(tx % 4) as usize] as f32) < light * light * 11.0 {
                0.045
            } else {
                0.0
            }
        }
        BackdropStyle::Grid => {
            if tx == 0 || ty == 0 {
                0.09
            } else {
                0.0
            }
        }
        BackdropStyle::GameBoy => {
            // a dot three art pixels square, a pixel of seam right and below
            if tx % 4 == 3 || ty % 4 == 3 {
                -0.35
            } else {
                0.06
            }
        }
        BackdropStyle::ThreeDs => {
            if (tx + ty) % 10 < 5 {
                0.06
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// How far the art pixel (`ax`, `ay`) of a whole canvas `aw` by `ah` art pixels lifts the
/// colour, for the patterns that are a picture rather than a tile.
fn picture_lift(style: BackdropStyle, ax: u32, ay: u32, aw: u32, ah: u32) -> f32 {
    let down = ay as f32 / ah.max(1) as f32;
    match style {
        // the DS Lite's menu: every other line a shade up, the whole a little lighter on top
        BackdropStyle::Ds => {
            let fade = 0.1 * (1.0 - down);
            fade + if ay % 2 == 0 { 0.05 } else { 0.0 }
        }
        // the PSP's: two ribbons rising and falling across, each a soft band with a bright edge
        BackdropStyle::Psp => {
            let fade = 0.08 * (1.0 - down);
            // across the canvas once, so the picture meets itself where it wraps
            let x = (ax % aw.max(1)) as f32 / aw.max(1) as f32;
            let tau = std::f32::consts::TAU;
            let ribbon = |mid: f32, amp: f32, waves: f32, phase: f32, width: f32| {
                let at = mid + amp * (tau * waves * x + phase).sin();
                let off = (down - at) / width;
                if (0.0..1.0).contains(&off) {
                    let edge = if off < 0.04 { 0.16 } else { 0.0 };
                    edge + 0.07 * (1.0 - off)
                } else {
                    0.0
                }
            };
            fade + ribbon(0.58, 0.06, 1.0, 0.6, 0.12) + ribbon(0.66, 0.05, 2.0, 2.4, 0.08)
        }
        // the Switch's: rounded tiles 36 art pixels square in a grid of 40, a rim round
        // each, the whole a little lighter on top
        BackdropStyle::Switch => {
            let fade = 0.06 * (1.0 - down);
            let (tx, ty) = (ax % 40, ay % 40);
            let inside = |x: u32, y: u32| {
                let (x, y) = (x as f32 - 19.5, y as f32 - 19.5);
                let (qx, qy) = (x.abs() - 12.0, y.abs() - 12.0);
                let corner = qx.max(0.0).hypot(qy.max(0.0));
                corner + qx.max(qy).min(0.0) - 6.0
            };
            let d = inside(tx, ty);
            fade + if d > 0.0 {
                0.0
            } else if d > -1.0 {
                0.1
            } else {
                0.035
            }
        }
        _ => 0.0,
    }
}

/// Whether the bit at (`cx`, `cy`) of a tile is lit, and how brightly: a scatter that looks
/// random but is the same every time.
fn bit(cx: u32, cy: u32) -> Option<f32> {
    let mut h = cx.wrapping_mul(0x9e37_79b1) ^ cy.wrapping_mul(0x85eb_ca77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    match h % 100 {
        0..=5 => Some(0.07),
        6..=8 => Some(0.14),
        _ => None,
    }
}

/// The picture a drifting pattern is drawn from: the canvas and one period more, so sliding it
/// left by up to a period always covers the canvas. Nothing for a style with no pattern.
pub fn backdrop_pattern_face(style: BackdropStyle, colour: u8) -> Option<UndoFace> {
    let period = backdrop_period(style);
    let (w, h) = (out_w() + period, out_h());
    let base = backdrop_rgb(colour);
    let ground = tint(base, 0.0);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let px = match style {
                BackdropStyle::Bits => {
                    let cells = period / BIT;
                    match bit((x / BIT) % cells, (y / BIT) % cells) {
                        Some(lift) => tint(base, lift),
                        None => ground,
                    }
                }
                BackdropStyle::Solid | BackdropStyle::Wallpaper => return None,
                BackdropStyle::Ds | BackdropStyle::Psp | BackdropStyle::Switch => tint(
                    base,
                    picture_lift(style, x / ART, y / ART, out_w() / ART, h / ART),
                ),
                tiled => tint(base, tile_lift(tiled, (x / ART) % TILE, (y / ART) % TILE)),
            };
            rgba.extend_from_slice(&px);
        }
    }
    Some(UndoFace { rgba, w, h })
}

/// The pattern `face` (made by [`backdrop_pattern_face`], `w` wide), drifted `shift` left.
pub fn draw_pattern(face: (TexId, u32), shift: u32, out: &mut Vec<Draw>) {
    out.push(Draw::Tex {
        x: -(shift as f32),
        y: 0.0,
        w: face.1 as f32,
        h: out_h() as f32,
        tex: face.0,
        alpha: 1.0,
    });
}

pub fn draw_solid(colour: u8, out: &mut Vec<Draw>) {
    let [r, g, b] = backdrop_rgb(colour);
    out.push(Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: out_w() as f32,
        h: out_h() as f32,
        colour: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0],
    });
}
