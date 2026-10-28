//! The SLEEP screen, opened from Settings: how long the handheld may be left alone before it
//! goes to sleep, and how long it may sleep, closed lid, POWER press or idle alike, before it
//! powers itself off.

use cartridge_input::{Action, Btn};
use cartridge_store::{SLEEP_IDLE_CHOICES, SLEEP_OFF_CHOICES};

use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sel {
    /// The quiet before it sleeps.
    Idle,
    /// The sleep before it powers off.
    Off,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SleepScreen {
    pub sel: Sel,
}

impl Default for SleepScreen {
    fn default() -> Self {
        SleepScreen { sel: Sel::Idle }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Close,
    Idle(u8),
    Off(u8),
}

/// A number of minutes as the rows print it: NEVER for none.
pub fn minutes_text(min: u8) -> String {
    match min {
        0 => "NEVER".into(),
        m => format!("{m} MIN"),
    }
}

/// The choice after or before `at` among `choices`, stopping at either end; one that is not
/// among them (a hand-edited file) counts as the nearest below it.
fn step(choices: &[u8], at: u8, forward: bool) -> u8 {
    let i = choices.iter().rposition(|c| *c <= at).unwrap_or(0);
    let to = if forward {
        (i + 1).min(choices.len() - 1)
    } else {
        i.saturating_sub(1)
    };
    choices[to]
}

impl SleepScreen {
    pub fn input(&mut self, idle: u8, off: u8, action: Action) -> Outcome {
        let forward = match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu => return Outcome::Close,
            Action::GbaDown(Btn::Up | Btn::Down) => {
                self.sel = match self.sel {
                    Sel::Idle => Sel::Off,
                    Sel::Off => Sel::Idle,
                };
                return Outcome::Stay;
            }
            Action::GbaDown(Btn::Left) => false,
            Action::GbaDown(Btn::Right | Btn::A) => true,
            _ => return Outcome::Stay,
        };
        match self.sel {
            Sel::Idle => match step(&SLEEP_IDLE_CHOICES, idle, forward) {
                to if to != idle => Outcome::Idle(to),
                _ => Outcome::Stay,
            },
            Sel::Off => match step(&SLEEP_OFF_CHOICES, off, forward) {
                to if to != off => Outcome::Off(to),
                _ => Outcome::Stay,
            },
        }
    }

    pub fn view(&self, idle: u8, off: u8) -> ListView {
        let row = |label, min| ViewRow {
            label: RowLabel::Setting(label),
            value: Some(RowValue::Picked(minutes_text(min))),
            dim: false,
        };
        ListView {
            rows: vec![row("Sleep After", idle), row("Power Off After", off)],
            lit: Some(match self.sel {
                Sel::Idle => 0,
                Sel::Off => 1,
            }),
            status: vec!["ALSO WITH THE LID CLOSED".into()],
            legend: vec![LegendHint::Back, LegendHint::Change],
        }
    }
}
