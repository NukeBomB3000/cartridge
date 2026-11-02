//! The volume gauge as the frontend draws it, for a program it does not draw for: PPSSPP runs
//! on the screen by itself, and the Mali video driver of the SDL it ships puts the gauge over
//! its picture (System/psp.sh). The driver cannot call into the frontend, so
//! `cartridge --hud-faces FILE` draws every face the gauge can have, in the colours of the
//! backdrop colour the card is set to, and writes them for the driver to read.
//!
//! The file is little-endian: "CHUD", then the face's width and height in art pixels, the
//! screen pixels to an art pixel, how far down the screen the gauge stands, how many cells it
//! has, and how many faces follow; each face is its RGBA, row by row. The faces come by icon
//! ([`ICONS`], the speaker, the headphones, muted) and, within it, by lit cells, 0 to `cells`.

use std::io;
use std::path::Path;

use cartridge_store::{read_app_state, VOLUME_MAX};
use cartridge_ui::kit::gauge::lit_cells;
use cartridge_ui::kit::palette::palette;
use cartridge_ui::{Gauge, GaugeIcon, HudKind, SCREEN_MARGIN, SPRITE_PX};

pub const MAGIC: &[u8; 4] = b"CHUD";

/// The icons the volume gauge has, in the order of the file.
pub const ICONS: [GaugeIcon; 3] = [GaugeIcon::Volume, GaugeIcon::Headphones, GaugeIcon::Muted];

/// Every face of the volume gauge, drawn in the colours of backdrop colour `colour`.
pub fn render(colour: u8) -> Vec<u8> {
    let pal = palette(colour);
    let cells = HudKind::Volume.cells();
    let mut faces = Vec::new();
    let (mut w, mut h) = (0, 0);
    for icon in ICONS {
        for lit in 0..=cells {
            let face = Gauge {
                kind: HudKind::Volume,
                lit,
                icon,
            }
            .face(pal);
            (w, h) = (face.w, face.h);
            faces.extend_from_slice(&face.rgba);
        }
    }
    let mut out = Vec::with_capacity(28 + faces.len());
    out.extend_from_slice(MAGIC);
    for n in [
        w,
        h,
        SPRITE_PX,
        SCREEN_MARGIN as u32,
        cells,
        ICONS.len() as u32 * (cells + 1),
    ] {
        out.extend_from_slice(&n.to_le_bytes());
    }
    out.extend_from_slice(&faces);
    out
}

/// The cells lit for a volume level, as the gauge lights them.
pub fn lit(level: u8) -> u32 {
    lit_cells(level.into(), VOLUME_MAX.into(), HudKind::Volume.cells())
}

/// Writes the faces for the card at `root` to `out`, through a file of its own so a driver
/// reading it never sees half of it.
pub fn write(root: &Path, out: &Path) -> io::Result<()> {
    let colour = read_app_state(root).backdrop_colour;
    let tmp = out.with_extension("tmp");
    std::fs::write(&tmp, render(colour))?;
    std::fs::rename(&tmp, out)
}
