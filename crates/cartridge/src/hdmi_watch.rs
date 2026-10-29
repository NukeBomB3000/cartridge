//! A cable going into the HDMI port, or out of it, while cartridge runs. The display engine's
//! output is switched by `System/hdmi.sh` before the frontend starts, so a change of cable means
//! the frontend starts again: [`Hotplug::poll`] says when, and the app saves the game and ends
//! itself, as BaseOS starts it afresh.

use std::path::PathBuf;

use cartridge_gfx::hdmi_active;

use cartridge_ui::Millis;

const EXTCON: &str = "/sys/class/extcon/hdmi/state";
const OUTPUT: &str = "/sys/class/disp/disp/attr/sys";

/// How often the cable is looked at.
pub const HOTPLUG_POLL_MS: Millis = 1000;

/// How many looks in a row must see the cable out of step with the output: a connector
/// that is being wiggled in does not restart anything until it holds.
const HOLDS: u8 = 2;

pub struct Hotplug {
    extcon: PathBuf,
    output: PathBuf,
    at: Millis,
    off_step: u8,
}

/// Whether the extcon state says a cable is in: `HDMI=1`.
pub fn cable_in(state: &str) -> bool {
    state
        .trim()
        .strip_prefix("HDMI=")
        .is_some_and(|v| v.trim() == "1")
}

impl Default for Hotplug {
    fn default() -> Self {
        Hotplug::at(EXTCON.into(), OUTPUT.into())
    }
}

impl Hotplug {
    pub fn at(extcon: PathBuf, output: PathBuf) -> Self {
        Hotplug {
            extcon,
            output,
            at: 0,
            off_step: 0,
        }
    }

    /// Looks at the cable if it is time, and says whether the output has to change with it. A
    /// handheld with no HDMI port has neither file and never does.
    pub fn poll(&mut self, now: Millis) -> bool {
        if now < self.at {
            return false;
        }
        self.at = now + HOTPLUG_POLL_MS;
        let cable = std::fs::read_to_string(&self.extcon).is_ok_and(|s| cable_in(&s));
        let hdmi = std::fs::read_to_string(&self.output).is_ok_and(|s| hdmi_active(&s));
        if cable == hdmi {
            self.off_step = 0;
            return false;
        }
        self.off_step += 1;
        self.off_step >= HOLDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rig(cable: &str, output: &str) -> (tempfile::TempDir, Hotplug) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("extcon"), cable).unwrap();
        std::fs::write(dir.path().join("sys"), output).unwrap();
        let watch = Hotplug::at(dir.path().join("extcon"), dir.path().join("sys"));
        (dir, watch)
    }

    #[test]
    fn the_extcon_state_says_whether_a_cable_is_in() {
        assert!(cable_in("HDMI=1\n"));
        assert!(!cable_in("HDMI=0\n"));
        assert!(!cable_in(""));
    }

    #[test]
    fn a_cable_that_matches_the_output_restarts_nothing() {
        let (_dir, mut watch) = rig("HDMI=0\n", "lcd output\n");
        for t in 0..5 {
            assert!(!watch.poll(t * 1000));
        }
        let (_dir, mut watch) = rig("HDMI=1\n", "hdmi output mode(10)\n");
        assert!(!watch.poll(0));
    }

    #[test]
    fn a_cable_out_of_step_restarts_once_it_holds() {
        let (_dir, mut watch) = rig("HDMI=1\n", "lcd output\n");
        assert!(!watch.poll(0), "one look is a wiggle");
        assert!(!watch.poll(500), "not looked at again yet");
        assert!(watch.poll(1000));
        let (_dir, mut watch) = rig("HDMI=0\n", "hdmi output mode(10)\n");
        assert!(!watch.poll(0));
        assert!(watch.poll(1000), "a cable pulled out too");
    }

    #[test]
    fn a_wiggle_that_settles_resets_the_count() {
        let (dir, mut watch) = rig("HDMI=1\n", "lcd output\n");
        assert!(!watch.poll(0));
        std::fs::write(dir.path().join("extcon"), "HDMI=0\n").unwrap();
        assert!(!watch.poll(1000));
        std::fs::write(dir.path().join("extcon"), "HDMI=1\n").unwrap();
        assert!(!watch.poll(2000));
    }

    #[test]
    fn a_handheld_without_the_files_never_restarts() {
        let mut watch = Hotplug::at("/nonexistent/a".into(), "/nonexistent/b".into());
        for t in 0..4 {
            assert!(!watch.poll(t * 1000));
        }
    }
}
