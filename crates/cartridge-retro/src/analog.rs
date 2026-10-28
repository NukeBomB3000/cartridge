//! The sticks of a handheld that has them, as the cores read them: the device thread writes the
//! raw axes here as they move, the emulator thread reads them when a core asks for an analog
//! pad. A handheld with no stick never writes, and a core then takes the d-pad as its left stick.

use std::sync::atomic::{AtomicBool, AtomicI16, Ordering};

/// How far the sticks reach as the kernel reports them: -4096 to 4096.
const REACH: i32 = 4096;

static AXES: [AtomicI16; 4] = [
    AtomicI16::new(0),
    AtomicI16::new(0),
    AtomicI16::new(0),
    AtomicI16::new(0),
];
static PRESENT: AtomicBool = AtomicBool::new(false);
static ANALOG_CORE: AtomicBool = AtomicBool::new(false);
static GAME_OWNS: AtomicBool = AtomicBool::new(false);

/// Stick axis `i` (0 left across, 1 left down, 2 right across, 3 right down) moved to `value`,
/// as the kernel reports it.
pub fn set_axis(i: usize, value: i32) {
    if let Some(axis) = AXES.get(i) {
        axis.store(value.clamp(-REACH, REACH) as i16, Ordering::Relaxed);
        PRESENT.store(true, Ordering::Relaxed);
    }
}

/// Axis `i` as a libretro analog value, -32767 to 32767.
pub fn axis(i: usize) -> i16 {
    scale(
        AXES.get(i)
            .map_or(0, |a| i32::from(a.load(Ordering::Relaxed))),
    )
}

/// `raw`, as the kernel reports an axis, held to its reach and laid over libretro's.
fn scale(raw: i32) -> i16 {
    (raw.clamp(-REACH, REACH) * i32::from(i16::MAX) / REACH) as i16
}

/// Whether the handheld has said it has sticks: one has moved.
pub fn present() -> bool {
    PRESENT.load(Ordering::Relaxed)
}

/// A core that reads an analog pad is running (a DualShock), or is let go of.
pub fn set_analog_core(on: bool) {
    ANALOG_CORE.store(on, Ordering::Relaxed);
    if !on {
        GAME_OWNS.store(false, Ordering::Relaxed);
    }
}

pub fn analog_core() -> bool {
    ANALOG_CORE.load(Ordering::Relaxed)
}

/// Whether the game has the sticks to itself: a core that reads them is running and nothing is
/// open over it, so they do not also press the d-pad. Anywhere else they steer as the d-pad.
pub fn set_game_owns_sticks(on: bool) {
    GAME_OWNS.store(on, Ordering::Relaxed);
}

pub fn game_owns_sticks() -> bool {
    GAME_OWNS.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kernels_reach_is_the_whole_of_a_libretro_axis() {
        assert_eq!(scale(4096), i16::MAX);
        assert_eq!(scale(-4096), -i16::MAX);
        assert_eq!(scale(0), 0);
        assert!((i32::from(scale(2048)) - i32::from(i16::MAX) / 2).abs() < 8);
        assert_eq!(
            scale(9000),
            i16::MAX,
            "an axis past its reach was not held to it"
        );
    }
}
