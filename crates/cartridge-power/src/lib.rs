mod baseos;
mod device;
mod motor;
mod power;
mod refresh;
mod sim;

pub use baseos::{
    baseos_release, parse_baseos_release, read_baseos_release, BaseosRelease, BASEOS_RELEASE,
};
pub use device::{
    backlight_value, card_mount_in, has_bit, motor_change, motor_pin, read_only_in,
    record_first_frame, rumble_node, stage_repair, take_repair_result, trace_first_frame,
    uptime_seconds, CardMount, DevicePlatform,
};
pub use motor::Motor;
pub use power::Power;
pub use sim::SimPlatform;

use std::path::Path;
use std::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Charge {
    Unknown,
    Discharging,
    Charging,
    Full,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Battery {
    pub percent: u8,
    pub charge: Charge,
}

/// How the last card repair ended, as `repair.sh` left it for the boot after.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RepairResult {
    Repaired,
    Clean,
    Failed,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LedState {
    Off,
    Running,
    Low,
    Charging,
    Charged,
}

/// The backlight's top step, the brightness setting's.
pub const BACKLIGHT_STEPS: u8 = 20;

pub trait Platform: Send {
    /// The backlight at `step` of 0 to [`BACKLIGHT_STEPS`]: 0 is as dim as the panel still
    /// shows a picture, never dark.
    fn set_backlight(&mut self, step: u8);
    /// The backlight off, for the lid closed.
    fn backlight_off(&mut self) {
        self.set_backlight(0);
    }
    /// The CPU held to at most `khz`, or let run at its top with None.
    fn set_cpu_cap(&mut self, _khz: Option<u32>) {}
    /// The GPU held at its top clock, for a core that draws with it, or left to its governor.
    fn set_gpu_boost(&mut self, _on: bool) {}
    fn battery(&self) -> Option<Battery>;
    fn charge(&self) -> Charge;
    fn set_led(&mut self, state: LedState);
    fn poweroff(&mut self) -> !;
    fn restart(&mut self) -> !;
    fn root(&self) -> &Path;
    fn now(&self) -> i64;
    fn set_clock(&mut self, secs: i64);
    fn set_rumble(&mut self, strength: u16);

    fn relink_adb(&mut self) -> bool {
        false
    }

    /// Whether the panel is one known to run at 120 Hz, whose registers can be set.
    fn supports_high_refresh(&self) -> bool {
        false
    }

    /// Runs the panel at 120 Hz, or back at 60; whether it now does, read back.
    fn set_high_refresh(&mut self, _high: bool) -> bool {
        false
    }

    fn high_refresh_enabled(&self) -> bool {
        false
    }

    /// Puts back what the display lost while the lid was shut.
    fn resume_display(&mut self) {}

    fn headphones(&self) -> bool {
        false
    }

    /// Whether the card cartridge runs from is mounted read-only, as the kernel leaves a FAT
    /// card it found damaged.
    fn card_read_only(&self) -> bool {
        false
    }

    /// Hands the handheld to `repair.sh`, which unmounts the card, checks it and reboots.
    fn repair_card(&mut self) -> ! {
        self.restart()
    }

    /// How the repair before this boot ended, once.
    fn take_repair_result(&mut self) -> Option<RepairResult> {
        None
    }
}

pub trait LidPolicy {
    fn on_close(&mut self);
    fn on_open(&mut self);
    fn timeout(&self) -> Duration;
}
