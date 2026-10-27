//! The EQUALIZER screen, opened from Settings: five bands for whatever is playing out now, the
//! speaker's or the headphones', a preset to start from, and a test sound to hear them on.

use cartridge_input::{Action, Btn};
use cartridge_store::{EQ_BANDS, EQ_DB_MAX};

/// The presets Y steps through, by name.
pub const PRESETS: [(&str, [i8; EQ_BANDS]); 4] = [
    ("Flat", [0; EQ_BANDS]),
    // the handhelds' speakers: spared the low end they only rattle at, given the middle where
    // voices and melodies sit
    ("Speaker", [-4, 0, 2, 3, 1]),
    ("Bass", [6, 3, 0, 0, 0]),
    ("Clear", [-2, -1, 1, 3, 4]),
];

/// What the gains match among the presets, or None for a hand-made setting.
pub fn preset_of(gains: [i8; EQ_BANDS]) -> Option<&'static str> {
    PRESETS.iter().find(|(_, g)| *g == gains).map(|(n, _)| *n)
}

/// The band named for a column, under it: its middle in Hz, kHz past a thousand.
pub fn band_name(hz: f32) -> String {
    if hz >= 1000.0 {
        let k = hz / 1000.0;
        if k.fract() == 0.0 {
            format!("{k:.0}K")
        } else {
            format!("{k:.1}K")
        }
    } else {
        format!("{hz:.0}")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EqualizerScreen {
    /// The band the cursor is on.
    pub band: usize,
    /// The test sound is playing.
    pub playing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Close,
    Changed([i8; EQ_BANDS]),
}

impl EqualizerScreen {
    pub fn input(&mut self, gains: [i8; EQ_BANDS], action: Action) -> Outcome {
        let mut next = gains;
        match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu => {
                self.playing = false;
                return Outcome::Close;
            }
            Action::GbaDown(Btn::Left) => self.band = self.band.saturating_sub(1),
            Action::GbaDown(Btn::Right) => self.band = (self.band + 1).min(EQ_BANDS - 1),
            Action::GbaDown(Btn::Up) => {
                next[self.band] = (gains[self.band] + 1).min(EQ_DB_MAX);
            }
            Action::GbaDown(Btn::Down) => {
                next[self.band] = (gains[self.band] - 1).max(-EQ_DB_MAX);
            }
            Action::GbaDown(Btn::A) => self.playing = !self.playing,
            Action::GbaDown(Btn::Y) => {
                let at = PRESETS.iter().position(|(_, g)| *g == gains);
                let to = at.map_or(0, |i| (i + 1) % PRESETS.len());
                next = PRESETS[to].1;
            }
            _ => {}
        }
        if next == gains {
            Outcome::Stay
        } else {
            Outcome::Changed(next)
        }
    }
}

/// What the panel shows, for baking it again only when something on it changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EqView {
    pub gains: [i8; EQ_BANDS],
    pub band: usize,
    pub playing: bool,
    pub headphones: bool,
}
