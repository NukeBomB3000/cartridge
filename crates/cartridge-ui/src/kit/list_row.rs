//! One Wi-Fi network or Bluetooth device in a list, as assets/ui/components/cartridge-list-row.js:
//! a row (row.rs) with its name on the left in its own case and, on the right, what it is to
//! the handheld and its icons: a network's padlock if it needs a password and how strong it
//! comes in, or the kind of device it is.
//!
//! The icons stand on the capitals' rows. A network's padlock is 5 art pixels wide like a letter
//! and its bars 7, 3 pixels apart and a letter's width (6) clear of the value, so they read as a
//! pair of their own; their room is kept when a row has none, so every row's value, padlock and
//! bars line up down the list. A device's row keeps no room for either: its icon stands where
//! the bars end, in the middle of a room 9 pixels wide (the widest icon's width) a letter's
//! width clear of the value, so a list of devices has its icons on one middle column and its
//! values lined up. The icons are drawn in the name's colour. Everything else is a row's:
//! `ROW_H` tall, placed in a panel's well by `row_rect`, the same band, words and colours.

use super::palette::Palette;
use super::pixel::text_width;
use super::row::{art, base, colour, inked, line, name, word, Ink, Item};
use crate::plate::UndoFace;

/// The top bar's Wi-Fi icon (WIFI in status_bar.rs), its three parts numbered in the order they
/// light: 1 the dot, 2 the small arc, 3 the big one.
#[rustfmt::skip]
const SIGNAL: [&str; 7] = [
    "..333..",
    ".3...3.",
    "3.....3",
    "..222..",
    ".2...2.",
    ".......",
    "...1...",
];

/// A padlock as tall as the capitals and as wide as a letter: the shackle, then the body with
/// its keyhole.
#[rustfmt::skip]
const LOCK: [&str; 7] = [
    ".###.",
    "#...#",
    "#...#",
    "#####",
    "##.##",
    "##.##",
    "#####",
];

/// How far the padlock stays clear of the value, and the bars of the padlock.
const LOCK_GAP: i32 = 6;
const SIGNAL_GAP: i32 = 3;

/// Headphones, as tall as the capitals: the band over the head and two ear cups, each two
/// pixels wide and three tall.
#[rustfmt::skip]
const AUDIO: [&str; 7] = [
    "..###..",
    ".#...#.",
    "#.....#",
    "#.....#",
    "##...##",
    "##...##",
    "##...##",
];

/// A gamepad: its body with the d-pad's cross cut out on the left and a button on the right, as
/// far in from the right as the cross's middle is from the left, and the grips under them.
#[rustfmt::skip]
const GAMEPAD: [&str; 7] = [
    ".#######.",
    "##.######",
    "#...##.##",
    "##.######",
    "#########",
    "###...###",
    "##.....##",
];

/// A keyboard: a block with two rows of keys cut out of it, the second half a key along, and
/// the space bar under them.
#[rustfmt::skip]
const KEYBOARD: [&str; 7] = [
    "#########",
    "#.#.#.#.#",
    "#########",
    "##.#.#.##",
    "#########",
    "##.....##",
    "#########",
];

/// Any other device: the Bluetooth rune, its stem with a stroke from either end out to a point
/// on its right and from there across the stem to its left.
#[rustfmt::skip]
const OTHER: [&str; 7] = [
    "..##.",
    "#.#.#",
    ".###.",
    "..#..",
    ".###.",
    "#.#.#",
    "..##.",
];

/// How wide a device's icon's room is, the gamepad's and the keyboard's width, each icon in the
/// middle of it; and how far it stays clear of the value, a letter's width like the padlock.
pub const DEVICE_W: u32 = 9;
const DEVICE_GAP: i32 = 6;

/// The most bars a signal lights.
pub const BARS: u8 = 3;

/// What a Bluetooth device is, for the icon its row shows.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    /// Headphones, a headset or a speaker.
    Audio,
    Gamepad,
    Keyboard,
    /// Anything else, or a device that does not say.
    Other,
}

impl DeviceKind {
    /// The kind's icon, as tall as the capitals and at most `DEVICE_W` wide, its '#' drawn in
    /// the name's colour.
    pub fn icon(self) -> &'static [&'static str; 7] {
        match self {
            DeviceKind::Audio => &AUDIO,
            DeviceKind::Gamepad => &GAMEPAD,
            DeviceKind::Keyboard => &KEYBOARD,
            DeviceKind::Other => &OTHER,
        }
    }
}

/// One network or device, as cartridge-list-row's attributes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ListRow<'a> {
    /// The network's or the device's name, from the left, in its own case (names are the
    /// player's).
    pub label: &'a str,
    /// What it is to the handheld, from the right: CONNECTED, JOINING, SAVED for a network;
    /// CONNECTED, CONNECTING, PAIRING, PAIRED for a device. Optional.
    pub value: Option<&'a str>,
    /// 0 to 3: the Wi-Fi bars, that many of the three lit and the rest drawn faint (more than 3
    /// is 3). Without it there are no bars, and their room is kept.
    pub signal: Option<u8>,
    /// A padlock beside the bars, for a network with a password; its room is kept without.
    pub lock: bool,
    /// A Bluetooth device's kind: the row is a device's, its icon in the middle of a room
    /// `DEVICE_W` wide where a network's bars end, and no room kept for a padlock or bars
    /// (`signal` and `lock` are not drawn).
    pub device: Option<DeviceKind>,
    /// The row under the cursor, as a `Row`'s.
    pub lit: bool,
    /// A network that cannot be joined, as a `Row`'s.
    pub dim: bool,
}

/// The bars `level` of the way lit: the parts reached in `on`, the rest in `off`.
fn bars(level: u8, on: char, off: char) -> Vec<String> {
    SIGNAL
        .iter()
        .map(|row| {
            row.chars()
                .map(|c| match c.to_digit(10) {
                    Some(part) if part <= u32::from(level) => on,
                    Some(_) => off,
                    None => c,
                })
                .collect()
        })
        .collect()
}

/// A network's or a device's row `width_u` art pixels wide and `ROW_H` tall, drawn as
/// cartridge-list-row draws `row`.
pub fn list_row_face(width_u: u32, row: &ListRow, pal: &Palette) -> UndoFace {
    let ink = Ink::of(row.lit, row.dim);
    let value = row.value.filter(|v| !v.is_empty());

    // left to right after the name: the value, handing its last blank pixel back as a row's
    // does, then a device's icon, or a network's padlock and bars, whose room is always there
    let mut items = Vec::new();
    if let Some(value) = value {
        items.push(Item {
            w: text_width(value),
            before: 0,
            after: -1,
        });
    }
    if row.device.is_some() {
        items.push(Item {
            w: DEVICE_W,
            before: DEVICE_GAP,
            after: 0,
        });
    } else {
        let lock = Item {
            w: LOCK[0].len() as u32,
            before: LOCK_GAP,
            after: 0,
        };
        let signal = Item {
            w: SIGNAL[0].len() as u32,
            before: SIGNAL_GAP,
            after: 0,
        };
        items.extend([lock, signal]);
    }
    let (room, xs) = line(width_u, &items);
    let mut xs = xs.into_iter();

    let mut face = base(width_u, row.lit, pal);
    name(&mut face, row.label, room, colour(pal, ink.name));
    if let Some(value) = value {
        let x = xs.next().expect("the value's place");
        word(&mut face, value, x, colour(pal, ink.value));
    }
    if let Some(kind) = row.device {
        let icon = kind.icon();
        let x = xs.next().expect("the icon's place");
        let x = x + (DEVICE_W - icon[0].len() as u32) as i32 / 2;
        art(&mut face, &inked(icon, ink.name), x, pal);
        return face;
    }
    let (lock_x, signal_x) = (xs.next(), xs.next());
    if row.lock {
        let x = lock_x.expect("the padlock's place");
        art(&mut face, &inked(&LOCK, ink.name), x, pal);
    }
    if let Some(level) = row.signal {
        let x = signal_x.expect("the bars' place");
        art(&mut face, &bars(level.min(BARS), ink.name, ink.off), x, pal);
    }
    face
}
