//! The link screen's art, as assets/ui/components/cartridge-link.js draws it: the link port along the
//! console's top edge, the GBA link cable's plug on its cable, the wireless adapter, the radio
//! arcs it calls out with and the clicks of a plug going home. Each piece is a sprite written a
//! character to an art pixel, baked one texel to an art pixel and drawn at 2x like the kit's
//! faces; link_screen.rs in cartridge moves them.
//!
//! The plug and the adapter are real things and keep their own colours, the plug the purple of
//! the GBA's link cable; the port, the arcs and the clicks are drawn in the kit's palette, so they
//! follow Settings > Theme > Colour.

use crate::kit::palette::Palette;
use crate::kit::pixel::{blank, fill, flip, letters, ADVANCE};
use crate::plate::UndoFace;

/// A piece's own colours, by the keys its rows are written in: o its outline, D the dark of a
/// slot, S shadow, B its body, G lit, W the brightest edge. Lit from the top left.
type Own = [(char, [u8; 3]); 6];

const fn ramp([o, d, s, b, g, w]: [u32; 6]) -> Own {
    [
        ('o', hex(o)),
        ('D', hex(d)),
        ('S', hex(s)),
        ('B', hex(b)),
        ('G', hex(g)),
        ('W', hex(w)),
    ]
}

const fn hex(rgb: u32) -> [u8; 3] {
    [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]
}

/// The host's plug: the purple of the real cable.
const PURPLE: Own = ramp([0x1d1430, 0x2e2352, 0x46397a, 0x6a55a6, 0x8f7bd0, 0xb7a8ea]);
/// The joiner's plug, and the cable.
const GREY: Own = ramp([0x1f2024, 0x3a3b40, 0x6f7078, 0x9c9ca4, 0xc2c2c9, 0xe2e2e7]);
/// The adapter's smoky grey plastic.
const SMOKE: Own = ramp([0x15171a, 0x272a2f, 0x3a3f46, 0x4f5660, 0x6e7681, 0x9aa2ac]);

/// The host's plug, the cable's narrow end: the neck the cable goes into with its two rings, the
/// housing with three grip ridges and an emblem, and the tip with its slot, which goes into the
/// port.
#[rustfmt::skip]
const PLUG_HOST: [&str; 54] = [
    "..........oGBBBBBBSo..........",
    "..........oGBBBBBBSo..........",
    "..........oSSSSSSSSo..........",
    "..........oGBBBBBBSo..........",
    "..........oSSSSSSSSo..........",
    ".........oGBBBBBBBBSo.........",
    "...oooooooooooooooooooooooo...",
    ".ooGGGGGGGGGGGGGGGGGGGGGGGSoo.",
    ".oWGBBBBBBBBBBBBBBBBBBBBBBSSo.",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBSSSSSSSSSSSSSSSSSSBBBSSo",
    "oGWGBBGGGGGGGGGGGGGGGGGGBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBSSSSSSSSSSSSSSSSSSBBBSSo",
    "oGWGBBGGGGGGGGGGGGGGGGGGBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBSSSSSSSSSSSSSSSSSSBBBSSo",
    "oGWGBBGGGGGGGGGGGGGGGGGGBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBSSSSSSSSSSBBBBBBBSSo",
    "oGWGBBBBBSBBBBBBBBBBGBBBBBBSSo",
    "oGWGBBBBBSBBBBBBBBBBGBBBBBBSSo",
    "oGWGBBBBBSBBBBBBBBBBGBBBBBBSSo",
    "oGWGBBBBBBGGGGGGGGGGBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oBSSSSSSSSSSSSSSSSSSSSSSSSSSSo",
    ".oSSSSSSSSSSSSSSSSSSSSSSSSSSo.",
    ".ooSSSSSSSSSSSSSSSSSSSSSSSSoo.",
    "...oooooooooooooooooooooooo...",
    "......oDDDDDDDDDDDDDDDDo......",
    "......oGBBBBBBBBBBBBBBSo......",
    "......oGBBBBBBBBBBBBBBSo......",
    "......oGBSSSSSSSSSSSSBSo......",
    "......oGBDDDDDDDDDDDDBSo......",
    "......oGBDDDDDDDDDDDDBSo......",
    "......oGBDDDDDDDDDDDDBSo......",
    "......oGBGGGGGGGGGGGGBSo......",
    "......oGBBBBBBBBBBBBBBSo......",
    "......oGBBBBBBBBBBBBBBSo......",
    "......oSSSSSSSSSSSSSSSSo......",
    ".......oooooooooooooooo.......",
];

/// The joiner's plug, the cable's wide end, with the socket a third player's cable goes into.
#[rustfmt::skip]
const PLUG_JOIN: [&str; 54] = [
    ".............oGBBBBBBSo.............",
    ".............oGBBBBBBSo.............",
    ".............oSSSSSSSSo.............",
    ".............oGBBBBBBSo.............",
    ".............oSSSSSSSSo.............",
    "............oGBBBBBBBBSo............",
    "...oooooooooooooooooooooooooooooo...",
    ".ooGGGGGGGGGGGGGGGGGGGGGGGGGGGGGSoo.",
    ".oWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBSSSSSSSSSSSSSSSSSSSSSSSSBBBSSo",
    "oGWGBBGGGGGGGGGGGGGGGGGGGGGGGGBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBSSSSSSSSSSSSSSSSSSSSSSSSBBBSSo",
    "oGWGBBGGGGGGGGGGGGGGGGGGGGGGGGBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBSSSSSSSSSSSSSSSSSSSSSSSSBBBSSo",
    "oGWGBBGGGGGGGGGGGGGGGGGGGGGGGGBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBSSSSSSSSSSSSSSBBBBBBBBSSo",
    "oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo",
    "oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo",
    "oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo",
    "oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo",
    "oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo",
    "oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo",
    "oGWGBBBBBBBGGGGGGGGGGGGGGBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oBSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSo",
    ".oSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSo.",
    ".ooSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSoo.",
    "...oooooooooooooooooooooooooooooo...",
    ".........oDDDDDDDDDDDDDDDDo.........",
    ".........oGBBBBBBBBBBBBBBSo.........",
    ".........oGBBBBBBBBBBBBBBSo.........",
    ".........oGBSSSSSSSSSSSSBSo.........",
    ".........oGBDDDDDDDDDDDDBSo.........",
    ".........oGBDDDDDDDDDDDDBSo.........",
    ".........oGBDDDDDDDDDDDDBSo.........",
    ".........oGBGGGGGGGGGGGGBSo.........",
    ".........oGBBBBBBBBBBBBBBSo.........",
    ".........oGBBBBBBBBBBBBBBSo.........",
    ".........oSSSSSSSSSSSSSSSSo.........",
    "..........oooooooooooooooo..........",
];

/// The wireless adapter: a smoky dome over the antenna inside, a label plate, two screws, and
/// under it the connector that goes into the port.
#[rustfmt::skip]
const ADAPTER: [&str; 52] = [
    "....................oooooooooooooooooooooooooooooooooooooooo....................",
    "................ooooGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGSSoooo................",
    ".............oooGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSooo.............",
    "...........ooGBBBBBDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDBBBBSSoo...........",
    "..........oGBBBDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDBBSSo..........",
    ".........oGBBDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDBSSo.........",
    "........oGWGDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDBSSo........",
    "........oGWGDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDBSSo........",
    "........oGWGDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDBSSo........",
    "........oGWGDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDBSSo........",
    ".......oGWGDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDBSSo.......",
    ".......oGWGDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDBSSo.......",
    ".......oGWGDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDBSSo.......",
    ".......oGWGSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSBSSo.......",
    "......oGWGDDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDDBSSo......",
    "......oGWGDDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDDBSSo......",
    "......oGWGDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDBSSo......",
    "......oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo......",
    ".....oSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSo.....",
    ".....oGWGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGSSo.....",
    ".....oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.....",
    ".....oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.....",
    "....oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo....",
    "....oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo....",
    "....oGWGBBBBBBBBBBBBBBBDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDBBBBBBBBBBBBBBBBBSSo....",
    "....oGWGBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBSSo....",
    "...oGWGBBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBBSSo...",
    "...oGWGBBBoooBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBoooBBBBSSo...",
    "...oGWGBBoGSGoBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBoGSGoBBBSSo...",
    "...oGWGBBoSSSoBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBoSSSoBBBSSo...",
    "..oGWGBBBoGSGoBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBoGSGoBBBBSSo..",
    "..oGWGBBBBoooBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBoooBBBBBSSo..",
    "..oGWGBBBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBBBSSo..",
    "..oGWGBBBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBBBSSo..",
    ".oGWGBBBBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBBBBSSo.",
    ".oGWGBBBBBBBBBBBBBBBBBBBGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGBBBBBBBBBBBBBBBBBBBSSo.",
    ".oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.",
    ".oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo",
    "oGSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSo",
    "oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo",
    "................................oWGGGGGGGGGGGGSo................................",
    "................................oWGGGGGGGGGGGGSo................................",
    "................................oGooooooooooooSo................................",
    "................................oGoDDDDDDDDDDoSo................................",
    "................................oGooooooooooooSo................................",
    "................................oGGGGGGGGGGGGGSo................................",
    "................................oSSSSSSSSSSSSSSo................................",
    ".................................oooooooooooooo.................................",
];

/// The port's opening in the middle of the strip's top edge, its five pins in the frame's bright
/// colour, drawn over the strip.
#[rustfmt::skip]
const NOTCH: [&str; 8] = [
    "kkkkkkkkkkkkkkkkkkkkkk",
    ".kkkkkkkkkkkkkkkkkkkk.",
    "..kkkkkkkkkkkkkkkkkk..",
    "..kkHHkHHkHHkHHkHHkk..",
    "..kkHHkHHkHHkHHkHHkk..",
    "..kkkkkkkkkkkkkkkkkk..",
    "..kkkkkkkkkkkkkkkkkk..",
    "...kkkkkkkkkkkkkkkk...",
];

/// The clicks of a plug going home, either side of its housing over the port: the left half, the
/// right the same turned round.
#[rustfmt::skip]
const CLICKS_LEFT: [&str; 10] = [
    "...cc.....................",
    "....cc....................",
    ".....cc...................",
    "......cc..................",
    "..........................",
    "..........................",
    "..........................",
    "cccccc....................",
    "..........................",
    "..........................",
];

/// A row of the cable, in the grey plug's colours, repeated as long as the cable is.
const CABLE: &str = "oGBBSo";

/// Where CART's line goes on the adapter, its capitals two rows down it: in the label plate.
const ADAPTER_LABEL: (i32, i32) = (25, 24);

/// The strip's top edge, lit, over a stripe; under them the strip is the well colour.
const EDGE: [char; 2] = ['A', 's'];

/// The radio arcs on the adapter's right, two pixels thick, the smallest first.
#[rustfmt::skip]
const ARC_0: [&str; 14] = [
    "cc...",
    ".cc..",
    "..cc.",
    "..cc.",
    "...cc",
    "...cc",
    "...cc",
    "...cc",
    "...cc",
    "...cc",
    "..cc.",
    "..cc.",
    ".cc..",
    "cc...",
];

#[rustfmt::skip]
const ARC_1: [&str; 22] = [
    "cc.....",
    ".cc....",
    "..cc...",
    "...cc..",
    "...cc..",
    "....cc.",
    "....cc.",
    ".....cc",
    ".....cc",
    ".....cc",
    ".....cc",
    ".....cc",
    ".....cc",
    ".....cc",
    ".....cc",
    "....cc.",
    "....cc.",
    "...cc..",
    "...cc..",
    "..cc...",
    ".cc....",
    "cc.....",
];

#[rustfmt::skip]
const ARC_2: [&str; 30] = [
    "cc.......",
    ".cc......",
    "..cc.....",
    "...cc....",
    "...cc....",
    "....cc...",
    "....cc...",
    ".....cc..",
    ".....cc..",
    "......cc.",
    "......cc.",
    ".......cc",
    ".......cc",
    ".......cc",
    ".......cc",
    ".......cc",
    ".......cc",
    ".......cc",
    ".......cc",
    "......cc.",
    "......cc.",
    ".....cc..",
    ".....cc..",
    "....cc...",
    "....cc...",
    "...cc....",
    "...cc....",
    "..cc.....",
    ".cc......",
    "cc.......",
];

/// How tall a plug is, its neck to its tip, in art pixels.
pub const PLUG_H: u32 = PLUG_HOST.len() as u32;
/// How tall its tip is, the part that goes into the port, and how wide.
pub const TIP_H: u32 = 12;
pub const TIP_W: u32 = 18;
/// How wide the cable is.
pub const CABLE_W: u32 = CABLE.len() as u32;
/// The adapter's size, its connector the bottom `CONNECTOR_H` rows.
pub const ADAPTER_W: u32 = ADAPTER[0].len() as u32;
pub const ADAPTER_H: u32 = ADAPTER.len() as u32;
pub const CONNECTOR_H: u32 = 8;
/// How wide the port's opening is at the top of the strip.
pub const NOTCH_W: u32 = NOTCH[0].len() as u32;
/// The clicks' size: they stand either side of a plug's housing, just over the strip.
pub const CLICKS_W: u32 = 2 * CLICKS_LEFT[0].len() as u32;
pub const CLICKS_H: u32 = CLICKS_LEFT.len() as u32;
/// How many radio arcs there are on each side.
pub const RINGS: usize = 3;

/// Which end of the cable a plug is.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum End {
    /// The host's: the narrow end, in the real cable's purple.
    Host,
    /// The joiner's: the wide end, grey, with a socket for a third player's cable.
    Join,
}

/// Which side of the adapter an arc is on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// `rows` painted a texel to a character: a key `own` has in its own colour, the palette's keys
/// in `pal`, anything else (and '.') left clear.
fn drawn(rows: &[impl AsRef<str>], own: &[(char, [u8; 3])], pal: &Palette) -> UndoFace {
    let w = rows
        .iter()
        .map(|r| r.as_ref().chars().count())
        .max()
        .unwrap_or(0)
        .max(1) as u32;
    let mut face = blank(w, rows.len().max(1) as u32);
    for (y, row) in rows.iter().enumerate() {
        for (x, key) in row.as_ref().chars().enumerate() {
            let colour = own
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, c)| *c)
                .or_else(|| pal.key(key));
            if let Some(colour) = colour {
                fill(&mut face, [x as i32, y as i32, 1, 1], colour);
            }
        }
    }
    face
}

/// `rows` with `over` laid on them, its top left at (`x`, `y`): every character of `over` but
/// '.' takes the place of the one under it.
fn laid(rows: &[impl AsRef<str>], over: &[impl AsRef<str>], x: i32, y: i32) -> Vec<String> {
    rows.iter()
        .enumerate()
        .map(|(ry, row)| {
            let top = usize::try_from(ry as i32 - y)
                .ok()
                .and_then(|i| over.get(i))
                .map(|r| r.as_ref().chars().collect::<Vec<char>>());
            row.as_ref()
                .chars()
                .enumerate()
                .map(|(rx, k)| {
                    let at = usize::try_from(rx as i32 - x).ok();
                    match (&top, at) {
                        (Some(top), Some(at)) => top.get(at).copied().filter(|c| *c != '.'),
                        _ => None,
                    }
                    .unwrap_or(k)
                })
                .collect()
        })
        .collect()
}

/// The plug at `end` of the cable: the neck the cable goes into with its two rings, the housing
/// with its grip ridges, and the tip with its slot, `TIP_H` rows at the bottom, `TIP_W` wide in
/// the middle. The host's is 30 art pixels wide, the joiner's 36.
pub fn plug_face(end: End, pal: &Palette) -> UndoFace {
    match end {
        End::Host => drawn(&PLUG_HOST, &PURPLE, pal),
        End::Join => drawn(&PLUG_JOIN, &GREY, pal),
    }
}

/// A row of the cable: drawn as tall as the cable is long, each row the same.
pub fn cable_face(pal: &Palette) -> UndoFace {
    drawn(&[CABLE], &GREY, pal)
}

/// The adapter with CART on its label in Pixel Code, a face pixel to an art pixel, in the
/// lightest of its colours.
pub fn adapter_face(pal: &Palette) -> UndoFace {
    let ink: Vec<String> = letters("CART", ADVANCE)
        .iter()
        .map(|row| row.replace('#', "W"))
        .collect();
    let (x, y) = ADAPTER_LABEL;
    drawn(&laid(&ADAPTER, &ink, x, y), &SMOKE, pal)
}

/// The strip along the console's top edge, `w` by `h` art pixels: its lit edge and stripe, the
/// well colour under them, and the port's opening in the middle, rounded down to a whole pixel.
pub fn port_face(w: u32, h: u32, pal: &Palette) -> UndoFace {
    let rows: Vec<String> = (0..h as usize)
        .map(|y| {
            EDGE.get(y)
                .copied()
                .unwrap_or('w')
                .to_string()
                .repeat(w as usize)
        })
        .collect();
    let x = (w as i32 - NOTCH_W as i32).div_euclid(2);
    drawn(&laid(&rows, &NOTCH, x, 0), &[], pal)
}

/// Radio arc `ring` (0, the smallest, to 2) on `side` of the adapter.
pub fn arc_face(ring: usize, side: Side, pal: &Palette) -> UndoFace {
    let rows: &[&str] = match ring {
        0 => &ARC_0,
        1 => &ARC_1,
        _ => &ARC_2,
    };
    match side {
        Side::Right => drawn(rows, &[], pal),
        Side::Left => drawn(&flip(rows), &[], pal),
    }
}

/// The clicks of a plug going home: either side of its housing, over the strip.
pub fn clicks_face(pal: &Palette) -> UndoFace {
    let right = flip(&CLICKS_LEFT);
    let rows: Vec<String> = CLICKS_LEFT
        .iter()
        .zip(&right)
        .map(|(l, r)| format!("{l}{r}"))
        .collect();
    drawn(&rows, &[], pal)
}

/// Every piece, baked for `pal` and a screen `w` art pixels wide whose strip is `port_h` tall.
pub struct LinkArt {
    pub plug_host: UndoFace,
    pub plug_join: UndoFace,
    pub cable: UndoFace,
    pub adapter: UndoFace,
    pub port: UndoFace,
    pub arcs_right: [UndoFace; RINGS],
    pub arcs_left: [UndoFace; RINGS],
    pub clicks: UndoFace,
}

pub fn link_art(pal: &Palette, w: u32, port_h: u32) -> LinkArt {
    LinkArt {
        plug_host: plug_face(End::Host, pal),
        plug_join: plug_face(End::Join, pal),
        cable: cable_face(pal),
        adapter: adapter_face(pal),
        port: port_face(w, port_h, pal),
        arcs_right: [0, 1, 2].map(|i| arc_face(i, Side::Right, pal)),
        arcs_left: [0, 1, 2].map(|i| arc_face(i, Side::Left, pal)),
        clicks: clicks_face(pal),
    }
}
