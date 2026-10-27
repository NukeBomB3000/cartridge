use std::time::Duration;

use crate::{Battery, Charge, LedState, LidPolicy, Platform};

pub struct Power {
    platform: Box<dyn Platform>,
    level: u8,
    closed: bool,
    timeout: Duration,
    /// The CPU cap last written, once one has been.
    cpu_cap: Option<Option<u32>>,
    gpu_boost: Option<bool>,
}

impl Power {
    pub fn new(platform: Box<dyn Platform>, timeout: Duration) -> Self {
        Power {
            platform,
            level: 0,
            closed: false,
            timeout,
            cpu_cap: None,
            gpu_boost: None,
        }
    }

    /// The CPU held to at most `khz`, or let run at its top; only a change is written.
    pub fn set_cpu_cap(&mut self, khz: Option<u32>) {
        if self.cpu_cap != Some(khz) {
            self.cpu_cap = Some(khz);
            self.platform.set_cpu_cap(khz);
        }
    }

    /// The GPU held at its top clock or let go; only a change is written.
    pub fn set_gpu_boost(&mut self, on: bool) {
        if self.gpu_boost != Some(on) {
            self.gpu_boost = Some(on);
            self.platform.set_gpu_boost(on);
        }
    }

    pub fn set_backlight(&mut self, step: u8) {
        self.level = step;
        if !self.closed {
            self.platform.set_backlight(step);
        }
    }

    pub fn battery(&self) -> Option<Battery> {
        self.platform.battery()
    }

    pub fn charge(&self) -> Charge {
        self.platform.charge()
    }

    pub fn headphones(&self) -> bool {
        self.platform.headphones()
    }

    pub fn set_led(&mut self, state: LedState) {
        self.platform.set_led(state)
    }

    pub fn set_rumble(&mut self, strength: u16) {
        self.platform.set_rumble(strength);
    }

    pub fn relink_adb(&mut self) -> bool {
        self.platform.relink_adb()
    }

    pub fn supports_high_refresh(&self) -> bool {
        self.platform.supports_high_refresh()
    }

    pub fn set_high_refresh(&mut self, high: bool) -> bool {
        self.platform.set_high_refresh(high)
    }

    pub fn high_refresh_enabled(&self) -> bool {
        self.platform.high_refresh_enabled()
    }

    pub fn poweroff(&mut self) -> ! {
        self.platform.poweroff()
    }

    pub fn restart(&mut self) -> ! {
        self.platform.restart()
    }

    pub fn card_read_only(&self) -> bool {
        self.platform.card_read_only()
    }

    pub fn repair_card(&mut self) -> ! {
        self.platform.repair_card()
    }

    pub fn take_repair_result(&mut self) -> Option<crate::RepairResult> {
        self.platform.take_repair_result()
    }

    pub fn now(&self) -> i64 {
        self.platform.now()
    }

    pub fn set_clock(&mut self, secs: i64) {
        self.platform.set_clock(secs);
    }
}

impl LidPolicy for Power {
    fn on_close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.platform.backlight_off();
    }

    fn on_open(&mut self) {
        self.closed = false;
        self.platform.resume_display();
        self.platform.set_backlight(self.level);
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }
}
