//! The ACHIEVEMENTS screen, opened from Settings: a lone panel with ACHIEVEMENTS (on or off) and
//! ACCOUNT (SIGN IN, or the account), and two lines saying how the sign-in stands. The account is
//! typed on the keyboard screen; Y signs it out.

use cartridge_input::Action;
use cartridge_ui::kit::bar::Hint;
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::{Draw, SettingsRow, UndoFace};

use crate::app::{App, Phase};
use crate::cheevos_screen::{forget_account, keep_account, Outcome as CheevosOutcome};
use crate::list_view::{LegendHint, ListView};
use crate::nav::Dir;
use crate::screens::lone::{hold, notes, notes_height, rows_height, Lone, NOTE_GAP};

/// How wide the panel is, in art pixels.
pub const WIDTH: u32 = 280;

/// The panel's title.
pub const TITLE: &str = "ACHIEVEMENTS";

/// The note lines the panel keeps room for, whatever it says, so it holds still.
pub const NOTE_LINES: usize = 2;

/// The ACHIEVEMENTS panel holding `view`'s rows and its notes under them, in `pal`.
pub fn cheevos_face(view: &ListView, pal: &Palette) -> UndoFace {
    let rows = rows_height(view.rows.len());
    let content = rows + notes_height(NOTE_LINES);
    let mut panel = panel_face(WIDTH, panel_height(content), TITLE, true, pal);
    hold(&mut panel, &view.rows, view.lit, pal);
    let shown = &view.status[..view.status.len().min(NOTE_LINES)];
    notes(&mut panel, shown, rows + NOTE_GAP, pal);
    panel.face
}

/// What the bottom bar shows between its actions: Y to sign out.
pub fn cheevos_hints(legend: &[LegendHint]) -> Vec<Hint<'static>> {
    legend
        .iter()
        .filter_map(|hint| match hint {
            LegendHint::SignOut => Some(Hint {
                key: "Y",
                word: "LOG OUT",
            }),
            _ => None,
        })
        .collect()
}

impl App {
    pub(crate) fn cheevos_input(&mut self, action: Action) {
        let status = crate::cheevos::status();
        let user = match &status {
            crate::cheevos::Status::SignedIn(user) => user.clone(),
            _ => String::new(),
        };
        let root = self.root.clone();
        let mut typed = None;
        let Phase::Cheevos(screen) = &mut self.phase else {
            return;
        };
        let before = screen.clone();
        let outcome = screen.input(action, &user, &mut typed);
        // the keyboard slides in over the list and away again; from the user name to the
        // password it stays where it is
        match (before.typing.is_some(), screen.typing.is_some()) {
            (false, true) => self.slide_from(Phase::Cheevos(before), Dir::Deeper),
            (true, false) => self.slide_from(Phase::Cheevos(before), Dir::Back),
            _ => {}
        }
        match outcome {
            CheevosOutcome::Close => self.go(
                Phase::Settings {
                    row: SettingsRow::Cheevos,
                },
                Dir::Back,
            ),
            CheevosOutcome::Toggle => {
                self.state.cheevos = !self.state.cheevos;
                self.persist();
            }
            CheevosOutcome::SignIn => {
                let (Some(root), Some((user, password))) = (root, typed) else {
                    return;
                };
                match keep_account(&root, &user, &password) {
                    Ok(()) => crate::cheevos::sign_in_again(&root),
                    Err(e) => eprintln!("cartridge: cheevos: {e}"),
                }
            }
            CheevosOutcome::SignOut => {
                if let Some(root) = root {
                    forget_account(&root);
                    crate::cheevos::sign_in_again(&root);
                }
            }
            CheevosOutcome::Stay => {}
        }
    }

    /// The screen's list, while it is open and not taking an account.
    pub fn cheevos_view(&self) -> Option<ListView> {
        match &self.phase {
            Phase::Cheevos(screen) if screen.typing.is_none() => {
                Some(screen.view(&crate::cheevos::status(), self.online(), self.state.cheevos))
            }
            _ => None,
        }
    }

    /// The hints the bottom bar shows on the screen; none while it is not open, or is taking an
    /// account.
    pub fn cheevos_bar_hints(&self) -> Vec<Hint<'static>> {
        self.cheevos_view()
            .map(|view| cheevos_hints(&view.legend))
            .unwrap_or_default()
    }

    pub fn set_cheevos_panel(&mut self, panel: Lone) {
        self.cheevos_panel = Some(panel);
    }

    pub(crate) fn draw_cheevos(&self, out: &mut Vec<Draw>) {
        if let Some(panel) = &self.cheevos_panel {
            panel.draw(out);
        }
    }
}
