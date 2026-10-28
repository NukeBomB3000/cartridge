//! The SLEEP screen, opened from Settings: a lone panel with its two rows, SLEEP AFTER the
//! quiet it takes and POWER OFF AFTER the sleep, and a note under them.

use cartridge_input::Action;
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::{Draw, SettingsRow, UndoFace};

use crate::app::{App, Phase};
use crate::list_view::ListView;
use crate::nav::Dir;
use crate::screens::lone::{hold, notes, notes_height, rows_height, Lone, NOTE_GAP};
use crate::sleep_screen::Outcome as SleepOutcome;

/// How wide the panel is, in art pixels.
pub const WIDTH: u32 = 220;

/// The panel's title.
pub const TITLE: &str = "SLEEP";

/// The SLEEP panel holding `view`'s rows and note, in `pal`.
pub fn sleep_face(view: &ListView, pal: &Palette) -> UndoFace {
    let content = rows_height(view.rows.len()) + notes_height(view.status.len());
    let mut panel = panel_face(WIDTH, panel_height(content), TITLE, true, pal);
    hold(&mut panel, &view.rows, view.lit, pal);
    notes(
        &mut panel,
        &view.status,
        rows_height(view.rows.len()) + NOTE_GAP,
        pal,
    );
    panel.face
}

impl App {
    pub(crate) fn sleep_input(&mut self, action: Action) {
        let (idle, off) = (self.state.sleep_idle_min, self.state.sleep_off_min);
        let Phase::Sleep(screen) = &mut self.phase else {
            return;
        };
        match screen.input(idle, off, action) {
            SleepOutcome::Close => self.go(
                Phase::Settings {
                    row: SettingsRow::Sleep,
                },
                Dir::Back,
            ),
            SleepOutcome::Idle(min) => {
                self.state.sleep_idle_min = min;
                self.persist();
            }
            SleepOutcome::Off(min) => {
                self.state.sleep_off_min = min;
                self.persist();
            }
            SleepOutcome::Stay => {}
        }
    }

    /// The SLEEP screen's list, while it is open.
    pub fn sleep_view(&self) -> Option<ListView> {
        match &self.phase {
            Phase::Sleep(screen) => {
                Some(screen.view(self.state.sleep_idle_min, self.state.sleep_off_min))
            }
            _ => None,
        }
    }

    pub fn set_sleep_panel(&mut self, panel: Lone) {
        self.sleep_panel = Some(panel);
    }

    pub(crate) fn draw_sleep_screen(&self, out: &mut Vec<Draw>) {
        if let Some(panel) = &self.sleep_panel {
            panel.draw(out);
        }
    }
}
