//! The bar across the top of the shelf: what the handheld is connected to on the left, the
//! time in the middle and the battery on the right.
//!
//! Everything on it stands in one band, a sprite cell tall: the icons are drawn on the text's
//! own pixel grid, and the words hang so their capitals fill the same band. All of it is
//! printed in the ink its [`ChromeColours`] give, the palette's cream or a theme's white, so
//! the faces are baked in that ink and baked again when it changes. Every face is the kit's,
//! one texel to an art pixel, uploaded NEAREST and drawn at two screen pixels to one of its own
//! on even screen pixels.

use cartridge_gfx::{out_w, Draw, TexId};
use cartridge_power::Battery;

use crate::battery::{draw_gauge, gauge_width, FIGURE_AT};
use crate::chrome::ChromeColours;
use crate::footer::Printed;
use crate::hud::HUD_INK;
use crate::kit::pixel::{blank, letters, quad, ACCENT, ADVANCE};
use crate::kit::text::{AIR, WORD_H};
use crate::plate::UndoFace;
use crate::sprite::{cropped, sprite_face, Rows, CELL, CELL_PX, SPRITE_PX};

pub const TOP_BAR_H: f32 = 36.0;
const MARGIN: f32 = 24.0;
const GAP: f32 = 4.0 * SPRITE_PX as f32;
const RULE_H: f32 = 2.0;

/// What an item is drawn in while it is on, and while it is off, on a card whose theme.txt sets
/// the chrome: the white and grey the bar was printed in before it took the palette.
pub const LINK_ON: [u8; 3] = HUD_INK;
pub const LINK_OFF: [u8; 3] = [0x5e, 0x5d, 0x66];

#[rustfmt::skip]
const WIFI: Rows = [
    "..###..",
    ".#...#.",
    "#.....#",
    "..###..",
    ".#...#.",
    ".......",
    "...#...",
];

#[rustfmt::skip]
const BLUETOOTH: Rows = [
    "...#...",
    "...##..",
    ".#.#.#.",
    "..###..",
    ".#.#.#.",
    "...##..",
    "...#...",
];

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Link {
    Wifi,
    Bluetooth,
    Ssh,
}

impl Link {
    pub const ALL: [Link; 3] = [Link::Wifi, Link::Bluetooth, Link::Ssh];

    /// The icon's sprite. A cell this small has no room to cross an icon out, so off is told
    /// by its ink alone and the shape stays put.
    fn sprite(self) -> Option<&'static Rows> {
        match self {
            Link::Wifi => Some(&WIFI),
            Link::Bluetooth => Some(&BLUETOOTH),
            Link::Ssh => None,
        }
    }
}

/// What the handheld is connected to, as the bar shows it.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Links {
    pub wifi: bool,
    /// None on a handheld with no Bluetooth radio, which then shows no icon for it.
    pub bluetooth: Option<bool>,
    /// The handheld can be reached over SSH and SFTP.
    pub ssh: bool,
}

/// One item's face, on or off in the ink of `colours`, one texel to an art pixel: an icon one
/// cell square, or "ssh" in bold cut to its ink.
pub fn link_face(link: Link, on: bool, colours: &ChromeColours) -> UndoFace {
    let ink = colours.link_ink(on);
    match link.sprite() {
        Some(rows) => sprite_face(rows, ink),
        None => ssh_face(ink),
    }
}

/// "ssh", in its own case, in Pixel Code drawn bold the kit's way: each letter a face pixel to
/// an art pixel and drawn again a pixel to its right, a pixel of air between letters. The h
/// stands a cell tall, the same as the icons, and the word has no descender, so its ink fills
/// the band exactly.
fn ssh_face(ink: [u8; 3]) -> UndoFace {
    let rows = letters("ssh", ADVANCE + 1);
    let w = rows[0].len();
    let mut face = blank(w as u32, rows.len() as u32);
    for (y, row) in rows.iter().enumerate() {
        let row = row.as_bytes();
        for x in 0..w {
            if row[x] == b'#' || (x > 0 && row[x - 1] == b'#') {
                let i = (y * w + x) * 4;
                face.rgba[i..i + 4].copy_from_slice(&[ink[0], ink[1], ink[2], 255]);
            }
        }
    }
    cropped(face)
}

/// A face uploaded, and its size in art pixels.
pub type Face = (TexId, u32, u32);

/// Every item's faces, off first then on.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LinkFaces {
    pub wifi: [Face; 2],
    pub bluetooth: [Face; 2],
    pub ssh: [Face; 2],
}

impl LinkFaces {
    fn of(&self, link: Link, on: bool) -> Face {
        let pair = match link {
            Link::Wifi => self.wifi,
            Link::Bluetooth => self.bluetooth,
            Link::Ssh => self.ssh,
        };
        pair[usize::from(on)]
    }
}

pub struct TopBar<'a> {
    /// What the bar and its rule are drawn in, the housing and the edge, and the ink the
    /// battery's capsule is drawn in. The faces handed in below are baked in the same ink.
    pub colours: ChromeColours,
    pub links: Links,
    pub faces: Option<&'a LinkFaces>,
    /// The time and the battery's figure, each a line of `kit::text::text_face` `w` art pixels
    /// wide.
    pub clock: Printed,
    pub battery: Option<Battery>,
    pub percent: Printed,
    pub bolt: Option<TexId>,
}

/// The top of the band every item stands in, centred in the bar above its rule.
fn band() -> f32 {
    ((TOP_BAR_H - RULE_H - CELL_PX) / 2.0).round()
}

/// `v` to the nearest even screen pixel, a half down: a whole art pixel.
fn on_grid(v: f32) -> f32 {
    let art = SPRITE_PX as f32;
    (v / art - 0.5).ceil() * art
}

/// How wide a word's ink is on the screen: its face, less the air after its last letter.
fn ink_w(word: Printed) -> f32 {
    (word.w.saturating_sub(AIR) * SPRITE_PX) as f32
}

impl TopBar<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let w = out_w() as f32;
        // no housing and no rule: the icons and figures stand straight on the background. An
        // empty plate keeps the bar's place in the draw list, where a slide moves it and the
        // layers over and under it keep their order.
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h: TOP_BAR_H - RULE_H,
            colour: [0.0; 4],
        });
        if let Some(faces) = self.faces {
            let mut x = MARGIN;
            for (link, on) in self.shown() {
                let (tex, fw, fh) = faces.of(link, on);
                let y = band() + (CELL.saturating_sub(fh) / 2 * SPRITE_PX) as f32;
                out.push(quad(tex, fw, fh, x, y));
                x += (fw * SPRITE_PX) as f32 + GAP;
            }
        }
        // the words' capitals stand in the band, ACCENT rows below the top of their faces
        let words = band() - (ACCENT as u32 * SPRITE_PX) as f32;
        if let Some(tex) = self.clock.face {
            let x = on_grid((w - ink_w(self.clock)) / 2.0);
            out.push(quad(tex, self.clock.w, WORD_H, x, words));
        }
        // the figure's ink, not its face, ends at the margin, as the Wi-Fi icon's starts at it
        let ink = self.colours.ink;
        match (self.battery, self.percent.face) {
            (Some(_), Some(tex)) => {
                let figure = w - MARGIN - ink_w(self.percent);
                let x = figure - FIGURE_AT;
                draw_gauge(
                    x,
                    band(),
                    self.battery,
                    Printed::default(),
                    self.bolt,
                    ink,
                    out,
                );
                out.push(quad(tex, self.percent.w, WORD_H, figure, words));
            }
            _ => {
                let x = w - MARGIN - gauge_width(Printed::default());
                draw_gauge(
                    x,
                    band(),
                    self.battery,
                    Printed::default(),
                    self.bolt,
                    ink,
                    out,
                );
            }
        }
    }

    /// The items along the left, in order, each with whether it is on.
    pub fn shown(&self) -> Vec<(Link, bool)> {
        let l = self.links;
        let mut items = vec![(Link::Wifi, l.wifi)];
        if let Some(on) = l.bluetooth {
            items.push((Link::Bluetooth, on));
        }
        items.push((Link::Ssh, l.ssh));
        items
    }
}
