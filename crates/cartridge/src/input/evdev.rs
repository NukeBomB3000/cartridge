use std::path::{Path, PathBuf};

use cartridge_input::{Btn, RawEvent};

pub const EVENT_BYTES: usize = 24;

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_SW: u16 = 0x05;
pub const SW_LID: u16 = 0x00;

pub const EV_ABS: u16 = 0x03;
pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;
pub const ABS_HAT0X: u16 = 0x10;
pub const ABS_HAT0Y: u16 = 0x11;
/// The sticks of a handheld that has them, the RG CubeXX's: the left across and down on Z and
/// RX, the right on RY and RZ, each -4096 to 4096.
pub const ABS_Z: u16 = 0x02;
pub const ABS_RX: u16 = 0x03;
pub const ABS_RY: u16 = 0x04;
pub const ABS_RZ: u16 = 0x05;

/// How far a stick is pushed before it presses its way on the d-pad, and how far back it must
/// come before it lets go, so a stick held near the edge does not chatter.
const STICK_PRESS: i32 = 2048;
const STICK_LET_GO: i32 = 1024;

/// The d-pad: the hat, and each stick pushed as far as a press.
#[derive(Default)]
pub struct Hat {
    x: i32,
    y: i32,
    sticks: [i32; 4],
}

impl Hat {
    pub fn feed(&mut self, ev: Ev) -> Vec<RawEvent> {
        let across = [Btn::Left, Btn::Right];
        let down = [Btn::Up, Btn::Down];
        let (held, ends, now, stick) = match ev.code {
            ABS_HAT0X => (&mut self.x, across, ev.value.signum(), false),
            ABS_HAT0Y => (&mut self.y, down, ev.value.signum(), false),
            ABS_Z | ABS_RX | ABS_RY | ABS_RZ => {
                let i = usize::from(ev.code - ABS_Z);
                // the cores that read an analog pad get the axis as it is
                cartridge_retro::analog::set_axis(i, ev.value);
                let held = &mut self.sticks[i];
                let reach = if *held == 0 {
                    STICK_PRESS
                } else {
                    STICK_LET_GO
                };
                let now = if ev.value.abs() >= reach {
                    ev.value.signum()
                } else {
                    0
                };
                (held, if i % 2 == 0 { across } else { down }, now, true)
            }
            _ => return Vec::new(),
        };
        let was = *held;
        if now == was {
            return Vec::new();
        }
        *held = now;
        // a game that reads the sticks has them to itself: they do not also press the d-pad
        if stick && cartridge_retro::analog::game_owns_sticks() {
            return Vec::new();
        }
        let btn = |v: i32| ends[usize::from(v > 0)];
        let mut out = Vec::new();
        if was != 0 {
            out.push(RawEvent::Up(btn(was)));
        }
        if now != 0 {
            out.push(RawEvent::Down(btn(now)));
        }
        out
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Ev {
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}

pub fn decode(bytes: &[u8]) -> Option<Ev> {
    let packet: &[u8; EVENT_BYTES] = bytes.get(..EVENT_BYTES)?.try_into().ok()?;
    Some(Ev {
        kind: u16::from_le_bytes([packet[16], packet[17]]),
        code: u16::from_le_bytes([packet[18], packet[19]]),
        value: i32::from_le_bytes([packet[20], packet[21], packet[22], packet[23]]),
    })
}

pub fn to_raw(ev: Ev) -> Option<RawEvent> {
    let btn = match ev.kind {
        EV_KEY => code_to_btn(ev.code)?,
        EV_SW if ev.code == SW_LID => Btn::Lid,
        _ => return None,
    };
    match ev.value {
        0 => Some(RawEvent::Up(btn)),
        1 => Some(RawEvent::Down(btn)),
        _ => None,
    }
}

pub fn code_to_btn(code: u16) -> Option<Btn> {
    Some(match code {
        0x130 => Btn::A,
        0x131 => Btn::B,
        0x132 => Btn::Y,
        0x133 => Btn::X,
        0x134 => Btn::L1,
        0x135 => Btn::R1,
        0x136 => Btn::Select,
        0x137 => Btn::Start,
        0x138 => Btn::Menu,
        0x13a => Btn::L2,
        0x13b => Btn::R2,
        0x139 => Btn::L3,
        0x13c => Btn::R3,
        115 => Btn::VolUp,
        114 => Btn::VolDown,
        116 => Btn::Power,
        _ => return None,
    })
}

const WANTED: [u16; 15] = [
    0x130, 0x131, 0x132, 0x133, 0x134, 0x135, 0x136, 0x137, 0x138, 0x13a, 0x13b, 0x162, 115, 114,
    116,
];

pub use cartridge_power::has_bit;

pub fn pick_devices(dev: &Path, sys: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dev) else {
        return Vec::new();
    };
    let mut nodes: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("event"))
        })
        .collect();
    nodes.sort();
    nodes.retain(|node| {
        let Some(name) = node.file_name() else {
            return false;
        };
        wanted_node(&sys.join(name).join("device"))
    });
    nodes
}

fn wanted_node(device: &Path) -> bool {
    let cap = |file: &str| std::fs::read_to_string(device.join("capabilities").join(file));
    let keys = cap("key").unwrap_or_default();
    let lid = cap("sw").is_ok_and(|sw| has_bit(&sw, SW_LID));
    lid || WANTED.iter().any(|bit| has_bit(&keys, *bit))
}

pub fn device_name(sys: &Path, node: &Path) -> String {
    node.file_name()
        .and_then(|n| std::fs::read_to_string(sys.join(n).join("device/name")).ok())
        .map(|n| n.trim().to_string())
        .unwrap_or_else(|| node.display().to_string())
}

/// Whose buttons a node's codes are: the handheld's own, which Anbernic numbered its own way, or
/// an external gamepad's, which follows Linux's standard layout.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Layout {
    Handheld,
    Pad,
}

/// A node on the Bluetooth bus, or one a Bluetooth gamepad's driver made (those that talk BLE
/// come through `uhid`), is an external pad.
pub fn layout_of(sys: &Path, node: &Path) -> Layout {
    let Some(name) = node.file_name() else {
        return Layout::Handheld;
    };
    let device = sys.join(name).join("device");
    let bus = std::fs::read_to_string(device.join("id/bustype")).unwrap_or_default();
    let real = std::fs::canonicalize(&device).unwrap_or_default();
    if bus.trim() == "0005" || real.to_string_lossy().contains("uhid") {
        Layout::Pad
    } else {
        Layout::Handheld
    }
}

/// An external pad's buttons on the Nintendo placing the handheld has: the east button is A and
/// the south one B, the north X and the west Y.
pub fn pad_code_to_btn(code: u16) -> Option<Btn> {
    Some(match code {
        0x131 => Btn::A,
        0x130 => Btn::B,
        0x133 => Btn::X,
        0x134 => Btn::Y,
        0x136 => Btn::L1,
        0x137 => Btn::R1,
        0x138 => Btn::L2,
        0x139 => Btn::R2,
        0x13a => Btn::Select,
        0x13b => Btn::Start,
        0x13c => Btn::Menu,
        0x13d => Btn::L3,
        0x13e => Btn::R3,
        // the volume keys of Bluetooth headphones
        115 => Btn::VolUp,
        114 => Btn::VolDown,
        _ => return None,
    })
}

pub fn to_raw_for(layout: Layout, ev: Ev) -> Option<RawEvent> {
    if layout == Layout::Handheld || ev.kind != EV_KEY {
        return to_raw(ev);
    }
    let btn = pad_code_to_btn(ev.code)?;
    match ev.value {
        0 => Some(RawEvent::Up(btn)),
        1 => Some(RawEvent::Down(btn)),
        _ => None,
    }
}

/// The reach of an axis, as the driver reports it: where it rests and how far it goes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Range {
    pub min: i32,
    pub max: i32,
}

impl Range {
    pub const STICK: Range = Range {
        min: -32768,
        max: 32767,
    };
    pub const TRIGGER: Range = Range { min: 0, max: 255 };

    /// `value` as the handheld's sticks give it, -4096 to 4096 with the rest at 0.
    pub fn stick(self, value: i32) -> i32 {
        let span = (self.max - self.min).max(1) as i64;
        let mid = (self.max as i64 + self.min as i64) / 2;
        let scaled = (value as i64 - mid) * 8192 / span;
        scaled.clamp(-4096, 4096) as i32
    }

    /// Whether a trigger reading is a press: past half of its way.
    pub fn pressed(self, value: i32) -> bool {
        (value as i64 - self.min as i64) * 2 > (self.max - self.min) as i64
    }
}

/// An external pad's axes: the two sticks go where the handheld's go, a trigger past half its
/// way presses L2 or R2.
pub struct PadAxes {
    pub sticks: [Range; 4],
    pub triggers: [Range; 2],
    held: [bool; 2],
}

impl PadAxes {
    pub fn new(sticks: [Range; 4], triggers: [Range; 2]) -> Self {
        PadAxes {
            sticks,
            triggers,
            held: [false; 2],
        }
    }

    pub fn feed(&mut self, hat: &mut Hat, ev: Ev) -> Vec<RawEvent> {
        // the left stick is the handheld's Z and RX, the right one its RY and RZ
        let stick = match ev.code {
            ABS_X => Some((0, ABS_Z)),
            ABS_Y => Some((1, ABS_RX)),
            ABS_RX => Some((2, ABS_RY)),
            ABS_RY => Some((3, ABS_RZ)),
            _ => None,
        };
        if let Some((i, code)) = stick {
            let value = self.sticks[i].stick(ev.value);
            return hat.feed(Ev {
                kind: ev.kind,
                code,
                value,
            });
        }
        let trigger = match ev.code {
            ABS_Z => 0,
            ABS_RZ => 1,
            _ => return hat.feed(ev),
        };
        let now = self.triggers[trigger].pressed(ev.value);
        if now == self.held[trigger] {
            return Vec::new();
        }
        self.held[trigger] = now;
        let btn = [Btn::L2, Btn::R2][trigger];
        vec![if now {
            RawEvent::Down(btn)
        } else {
            RawEvent::Up(btn)
        }]
    }
}
