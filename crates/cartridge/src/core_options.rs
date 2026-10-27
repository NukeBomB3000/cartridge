//! The pause menu SELECT + MENU opens over a DS game, after DSperate's own: PAUSED over the game
//! dimmed behind it, its rows saving and loading a state, the save states themselves, the core's
//! options, going back to the game and quitting it. Its OPTIONS page holds those of the core's
//! options a player has a use for, one to a row, Left and Right stepping the lit one through its
//! values; each change goes to the core at once and is kept in Config/core_options.ini, so the
//! next game on that core starts with it. B goes back a page, and out of the menu from its first;
//! MENU or SELECT + MENU close it.

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
pub const WIDTH: u32 = 220;

/// How dark the paused game stands behind the menu.
const DIM: f32 = 0.6;

/// The options kept for `core`, to set after its own defaults when a game loads.
pub fn kept_options(root: &Path, core: Core) -> Vec<(String, String)> {
    let mut kept: Vec<(String, String)> = ini::read(root, CORE_OPTIONS_FILE)
        .into_iter()
        .filter(|(key, _)| key.starts_with(core.as_str()))
        .collect();
    kept.sort();
    kept
}

/// The options of `core` its menu shows: DSperate's screen layout, the gap between its screens
/// and its frameskip, its buttons, stylus and engine being set here once; every option of any
/// other core.
pub fn shown(core: Core, option: &CoreOption) -> bool {
    match core {
        Core::Dsperate => matches!(
            option.key.as_str(),
            "dsperate_layout" | "dsperate_screen_gap" | "dsperate_frameskip"
        ),
        _ => true,
    }
}

/// The pause menu's rows, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseRow {
    SaveState,
    LoadState,
    States,
    Options,
    Resume,
    Quit,
}

impl PauseRow {
    pub const ALL: [PauseRow; 6] = [
        PauseRow::SaveState,
        PauseRow::LoadState,
        PauseRow::States,
        PauseRow::Options,
        PauseRow::Resume,
        PauseRow::Quit,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PauseRow::SaveState => "Save State",
            PauseRow::LoadState => "Load State",
            PauseRow::States => "Save States",
            PauseRow::Options => "Options",
            PauseRow::Resume => "Resume",
            PauseRow::Quit => "Quit",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Pause,
    Options,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoreOptionsScreen {
    pub title: String,
    pub page: Page,
    /// The lit row of the pause menu.
    pub row: usize,
    pub options: Vec<CoreOption>,
    /// The lit row of the options page.
    pub at: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Close,
    /// An option set to a new value.
    Set(String, String),
    SaveState,
    LoadState,
    States,
    Quit,
}

impl CoreOptionsScreen {
    pub fn input(&mut self, action: Action) -> Outcome {
        match action {
            Action::QuickMenu | Action::GameMenu => return Outcome::Close,
            Action::GbaDown(Btn::B) if self.page == Page::Options => {
                self.page = Page::Pause;
                return Outcome::Stay;
            }
            Action::GbaDown(Btn::B) => return Outcome::Close,
            _ => {}
        }
        if self.page == Page::Pause {
            return self.pause_input(action);
        }
        let n = self.options.len();
        match action {
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

    fn pause_input(&mut self, action: Action) -> Outcome {
        let n = PauseRow::ALL.len();
        match action {
            Action::GbaDown(Btn::Up) => self.row = (self.row + n - 1) % n,
            Action::GbaDown(Btn::Down) => self.row = (self.row + 1) % n,
            Action::GbaDown(Btn::A) => {
                return match PauseRow::ALL[self.row] {
                    PauseRow::SaveState => Outcome::SaveState,
                    PauseRow::LoadState => Outcome::LoadState,
                    PauseRow::States => Outcome::States,
                    PauseRow::Options => {
                        self.page = Page::Options;
                        self.at = 0;
                        Outcome::Stay
                    }
                    PauseRow::Resume => Outcome::Close,
                    PauseRow::Quit => Outcome::Quit,
                }
            }
            _ => {}
        }
        Outcome::Stay
    }

    /// What the menu's panel is titled on its page.
    pub fn page_title(&self) -> String {
        match self.page {
            Page::Pause => "PAUSED".into(),
            Page::Options => self.title.clone(),
        }
    }

    pub fn view(&self) -> ListView {
        if self.page == Page::Pause {
            return ListView {
                rows: PauseRow::ALL
                    .iter()
                    .map(|r| ViewRow {
                        label: RowLabel::Setting(r.label()),
                        value: None,
                        dim: false,
                    })
                    .collect(),
                lit: Some(self.row),
                status: vec!["A CHOOSE   B RESUME".into()],
                legend: vec![LegendHint::Back, LegendHint::Open],
            };
        }
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

    /// The core's options as it reports them, for the open screen: those a player has a use for.
    pub fn set_core_option_list(&mut self, options: Vec<CoreOption>) {
        let core = self.core();
        if let Some(screen) = self.core_options.as_mut() {
            screen.options = options.into_iter().filter(|o| shown(core, o)).collect();
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
            Outcome::SaveState => {
                self.core_options = None;
                self.save_state();
            }
            Outcome::LoadState => {
                self.core_options = None;
                self.load_newest();
            }
            Outcome::States => {
                self.core_options = None;
                self.open_polaroids();
            }
            Outcome::Quit => {
                self.core_options = None;
                self.eject();
            }
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
            .map(|s| (s.page_title(), s.view()))
    }

    pub fn set_core_options_panel(&mut self, panel: Lone) {
        self.core_options_panel = Some(panel);
    }

    /// The panel over the paused game, dimmed behind it.
    pub(crate) fn draw_core_options(&self, out: &mut Vec<Draw>) {
        if self.core_options.is_none() {
            return;
        }
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: cartridge_gfx::out_w() as f32,
            h: cartridge_gfx::out_h() as f32,
            colour: [0.0, 0.0, 0.0, DIM],
        });
        if let Some(panel) = &self.core_options_panel {
            panel.draw(out);
        }
    }
}
