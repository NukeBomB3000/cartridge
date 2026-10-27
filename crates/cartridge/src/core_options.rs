//! The core's own options, opened over a paused game with SELECT + MENU on a system that has no
//! link or disc screen there: every option the core declared, one to a row, Up and Down moving
//! between them and Left and Right stepping the lit one through its values. Each change goes to
//! the core at once and is kept in Config/core_options.ini, so the next game on that core starts
//! with it. B, MENU or SELECT + MENU again close it.

use std::path::Path;

use cartridge_input::{Action, Btn};
use cartridge_retro::CoreOption;
use cartridge_store::{ini, Core};
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::{Draw, UndoFace};

use crate::app::App;
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};
use crate::screens::lone::{hold, notes, notes_height, rows_height, Lone, NOTE_GAP};

/// Where the options a player changed are kept, by their keys, every core's in one file: each
/// core's keys start with its own name.
pub const CORE_OPTIONS_FILE: &str = "Config/core_options.ini";

/// How wide the panel is, in art pixels.
pub const WIDTH: u32 = 300;

/// The options kept for `core`, to set after its own defaults when a game loads.
pub fn kept_options(root: &Path, core: Core) -> Vec<(String, String)> {
    let mut kept: Vec<(String, String)> = ini::read(root, CORE_OPTIONS_FILE)
        .into_iter()
        .filter(|(key, _)| key.starts_with(core.as_str()))
        .collect();
    kept.sort();
    kept
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoreOptionsScreen {
    pub title: String,
    pub options: Vec<CoreOption>,
    pub at: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Close,
    /// An option set to a new value.
    Set(String, String),
}

impl CoreOptionsScreen {
    pub fn input(&mut self, action: Action) -> Outcome {
        let n = self.options.len();
        match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu | Action::GameMenu => Outcome::Close,
            Action::GbaDown(Btn::Up) if n > 0 => {
                self.at = (self.at + n - 1) % n;
                Outcome::Stay
            }
            Action::GbaDown(Btn::Down) if n > 0 => {
                self.at = (self.at + 1) % n;
                Outcome::Stay
            }
            Action::GbaDown(Btn::Left | Btn::Right) => {
                let forward = action == Action::GbaDown(Btn::Right);
                let Some(option) = self.options.get_mut(self.at) else {
                    return Outcome::Stay;
                };
                let count = option.values.len();
                if count < 2 {
                    return Outcome::Stay;
                }
                let i = option
                    .values
                    .iter()
                    .position(|v| *v == option.value)
                    .unwrap_or(0);
                let to = if forward {
                    (i + 1) % count
                } else {
                    (i + count - 1) % count
                };
                option.value = option.values[to].clone();
                Outcome::Set(option.key.clone(), option.value.clone())
            }
            _ => Outcome::Stay,
        }
    }

    pub fn view(&self) -> ListView {
        let rows = self
            .options
            .iter()
            .map(|o| ViewRow {
                label: RowLabel::Name(o.label.to_uppercase()),
                value: Some(RowValue::Picked(o.value.to_uppercase())),
                dim: o.values.len() < 2,
            })
            .collect();
        ListView {
            rows,
            lit: (!self.options.is_empty()).then_some(self.at),
            status: if self.options.is_empty() {
                vec!["THE CORE HAS NO OPTIONS".into()]
            } else {
                vec!["◀▶ CHANGE   B BACK".into()]
            },
            legend: vec![LegendHint::Back, LegendHint::Change],
        }
    }
}

/// The options panel holding `view`'s rows and its note under them, titled `title`, in `pal`.
pub fn core_options_face(title: &str, view: &ListView, pal: &Palette) -> UndoFace {
    let content = rows_height(view.rows.len()) + notes_height(view.status.len());
    let mut panel = panel_face(WIDTH, panel_height(content), title, true, pal);
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
    /// SELECT + MENU on a system with its core's options there: opens them over the paused game,
    /// their list filled in from the core on the next update.
    pub(crate) fn open_core_options(&mut self) {
        self.core_options = Some(CoreOptionsScreen {
            title: self.core().name().to_uppercase(),
            ..CoreOptionsScreen::default()
        });
    }

    /// Whether the options screen waits for the core's list.
    pub fn wants_core_options(&self) -> bool {
        self.core_options
            .as_ref()
            .is_some_and(|s| s.options.is_empty())
    }

    /// The core's options as it reports them, for the open screen.
    pub fn set_core_option_list(&mut self, options: Vec<CoreOption>) {
        if let Some(screen) = self.core_options.as_mut() {
            screen.options = options;
            screen.at = screen.at.min(screen.options.len().saturating_sub(1));
        }
    }

    /// An option changed on the screen, for the session to hand the core.
    pub fn take_core_option(&mut self) -> Option<(String, String)> {
        self.core_option_pending.take()
    }

    pub(crate) fn core_options_input(&mut self, action: Action) {
        let Some(screen) = self.core_options.as_mut() else {
            return;
        };
        match screen.input(action) {
            Outcome::Close => self.core_options = None,
            Outcome::Set(key, value) => {
                if let Some(root) = self.root.as_deref() {
                    if let Err(e) = ini::write(root, CORE_OPTIONS_FILE, &key, &value) {
                        eprintln!("cartridge: {CORE_OPTIONS_FILE}: {e}");
                    }
                }
                self.core_option_pending = Some((key, value));
            }
            Outcome::Stay => {}
        }
    }

    /// The options screen's title and list, while it is open.
    pub fn core_options_view(&self) -> Option<(String, ListView)> {
        self.core_options
            .as_ref()
            .map(|s| (s.title.clone(), s.view()))
    }

    pub fn set_core_options_panel(&mut self, panel: Lone) {
        self.core_options_panel = Some(panel);
    }

    /// The panel over the theme's backdrop, the game paused behind it.
    pub(crate) fn draw_core_options(&self, out: &mut Vec<Draw>) {
        if self.core_options.is_none() {
            return;
        }
        self.draw_background(out);
        if let Some(panel) = &self.core_options_panel {
            panel.draw(out);
        }
    }
}
