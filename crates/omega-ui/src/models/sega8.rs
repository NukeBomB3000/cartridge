//! Sega's 8-bit carts: the Game Gear's and the Master System's.
//!
//! Both are drawn 4 units to the millimetre. The Game Gear cart is 68 by 60 mm and 10 thick,
//! wider than it is tall, its top corners cut off, its label in a well across most of its face
//! and three grip ridges along its foot; the Master System cart is 100 by 134 mm and 20 thick,
//! its corners round, its label in a well over the top of its face and four grip ridges across
//! its foot over the step to its contacts. Both are black. The ridges and wells are relief in
//! the face, not geometry. The Master System's face is drawn about 1.8 pixels to the millimetre,
//! as the Mega Drive carts are; the Game Gear's, a handheld's small cart, about 4, so it stands
//! as tall as the carts beside it and its label reads.

use std::sync::OnceLock;

use crate::cart::{Board, Spec, PAD};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_svg, Detail};

const GAME_GEAR_SVG: &str = include_str!("../../assets/gamegear_cart.svg");
const GAME_GEAR_DETAIL_SVG: &str = include_str!("../../assets/gamegear_cart_detail.svg");
const MASTER_SYSTEM_SVG: &str = include_str!("../../assets/mastersystem_cart.svg");
const MASTER_SYSTEM_DETAIL_SVG: &str = include_str!("../../assets/mastersystem_cart_detail.svg");

/// Sega's 8-bit carts a model is built for.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Sega8Shell {
    GameGear,
    MasterSystem,
}

/// How thick each cart is, as a share of its width.
pub const GAME_GEAR_THICK: f32 = 10.0 / 68.0;
pub const MASTER_SYSTEM_THICK: f32 = 20.0 / 100.0;

/// How big the cart stands on the shelf, as a share of its own size: its face is made at the
/// size it stands at, so its label lands a texel to a pixel there.
pub const SHELF_SCALE: f32 = 1.0;

/// The Game Gear cart's outline, right half, in drawing units (272 by 240), clockwise from the
/// top: its cut top corner and its foot in two. The left half is its mirror.
const GAME_GEAR_HALF: &[(f32, f32)] = &[
    (256.0, 0.0),
    (272.0, 16.0),
    (272.0, 234.0),
    (270.24, 238.24),
    (266.0, 240.0),
];
const GAME_GEAR_UNITS: (f32, f32) = (272.0, 240.0);

/// The Master System cart's, right half, in drawing units (400 by 536): its round top corner in
/// three cuts and its foot in two.
const MASTER_SYSTEM_HALF: &[(f32, f32)] = &[
    (384.0, 0.0),
    (392.0, 2.14),
    (397.86, 8.0),
    (400.0, 16.0),
    (400.0, 528.0),
    (397.66, 533.66),
    (392.0, 536.0),
];
const MASTER_SYSTEM_UNITS: (f32, f32) = (400.0, 536.0);

/// The size of each face in pixels. Each is a multiple of four across, so it halves onto the art
/// grid in the middle of the row, and an even number down.
pub const GAME_GEAR_W: u32 = 280;
pub const GAME_GEAR_H: u32 = 246;
pub const MASTER_SYSTEM_W: u32 = 180;
pub const MASTER_SYSTEM_H: u32 = 242;

/// The label on each face, in its pixels: the inside of its well.
/// The Game Gear's label is ScreenScraper's scan's own shape, 236 by 172, so none of it is cut.
const GAME_GEAR_LABEL: (u32, u32, u32, u32) = (22, 21, 236, 172);
const MASTER_SYSTEM_LABEL: (u32, u32, u32, u32) = (16, 16, 148, 146);

/// How far in from the outline a clear shell's plastic lightens at its rim, in pixels.
const RIM: u32 = 5;

/// The cart's outline in drawing units, all the way round clockwise, and the drawing's size.
pub(crate) fn outline(shell: Sega8Shell) -> (Vec<(f32, f32)>, (f32, f32)) {
    let (half, units) = match shell {
        Sega8Shell::GameGear => (GAME_GEAR_HALF, GAME_GEAR_UNITS),
        Sega8Shell::MasterSystem => (MASTER_SYSTEM_HALF, MASTER_SYSTEM_UNITS),
    };
    let mut all = half.to_vec();
    all.extend(half.iter().rev().map(|&(x, y)| (units.0 - x, y)));
    (all, units)
}

/// How thick `shell` is, as a share of its width.
pub(crate) fn thick(shell: Sega8Shell) -> f32 {
    match shell {
        Sega8Shell::GameGear => GAME_GEAR_THICK,
        Sega8Shell::MasterSystem => MASTER_SYSTEM_THICK,
    }
}

/// The cart's silhouette `w` by `h`, as coverage: opaque inside the outline.
pub fn silhouette(shell: Sega8Shell, w: u32, h: u32) -> Vec<u8> {
    let svg = match shell {
        Sega8Shell::GameGear => GAME_GEAR_SVG,
        Sega8Shell::MasterSystem => MASTER_SYSTEM_SVG,
    };
    rasterise_svg(svg, w, h).unwrap_or_else(|| vec![255; (w * h) as usize])
}

/// The size of the cart's face in pixels.
pub(crate) fn face_box(shell: Sega8Shell) -> (u32, u32) {
    match shell {
        Sega8Shell::GameGear => (GAME_GEAR_W, GAME_GEAR_H),
        Sega8Shell::MasterSystem => (MASTER_SYSTEM_W, MASTER_SYSTEM_H),
    }
}

fn mask(shell: Sega8Shell) -> &'static [u8] {
    static GAME_GEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static MASTER_SYSTEM: OnceLock<Vec<u8>> = OnceLock::new();
    let lock = match shell {
        Sega8Shell::GameGear => &GAME_GEAR,
        Sega8Shell::MasterSystem => &MASTER_SYSTEM,
    };
    lock.get_or_init(|| {
        let (w, h) = face_box(shell);
        silhouette(shell, w, h)
    })
}

fn depth(shell: Sega8Shell) -> &'static [u8] {
    static GAME_GEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static MASTER_SYSTEM: OnceLock<Vec<u8>> = OnceLock::new();
    let lock = match shell {
        Sega8Shell::GameGear => &GAME_GEAR,
        Sega8Shell::MasterSystem => &MASTER_SYSTEM,
    };
    lock.get_or_init(|| {
        let (w, h) = face_box(shell);
        depth_map(mask(shell), w as usize, h as usize)
    })
}

fn detail(shell: Sega8Shell) -> &'static Detail {
    static GAME_GEAR: OnceLock<Detail> = OnceLock::new();
    static MASTER_SYSTEM: OnceLock<Detail> = OnceLock::new();
    let (lock, svg) = match shell {
        Sega8Shell::GameGear => (&GAME_GEAR, GAME_GEAR_DETAIL_SVG),
        Sega8Shell::MasterSystem => (&MASTER_SYSTEM, MASTER_SYSTEM_DETAIL_SVG),
    };
    lock.get_or_init(|| {
        let (w, h) = face_box(shell);
        rasterise_detail(svg, w, h).unwrap_or_else(|| Detail::blank(w, h))
    })
}

/// How the cart's face is laid out. A clear shell shows the board inside: across the cart, its
/// contacts along its foot.
pub(crate) fn spec(shell: Sega8Shell) -> Spec {
    let (w, h) = face_box(shell);
    let label = match shell {
        Sega8Shell::GameGear => GAME_GEAR_LABEL,
        Sega8Shell::MasterSystem => MASTER_SYSTEM_LABEL,
    };
    Spec {
        w,
        h,
        label,
        label_round: 0,
        mask: mask(shell),
        depth: depth(shell),
        detail: detail(shell),
        max_h: (label.3 - 2 * PAD) as f32,
        rim: RIM,
        board: Board {
            x: (0.1, 0.9),
            top: 0.12,
            pins: (0.15, 0.85),
            contacts_from: 0.9,
            traces_from: 0.82,
        },
    }
}
