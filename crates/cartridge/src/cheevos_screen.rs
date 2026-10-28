//! The ACHIEVEMENTS screen, opened from Settings: RetroAchievements on or off, and the player's
//! account on it, typed on the keyboard screen. The account is the player's to enter; it is kept
//! in `Config/retroachievements.txt`, the password only until the first sign-in has handed back a
//! token to keep in its place. Games are followed softcore only.

use std::path::Path;

use cartridge_cheevos::CHEEVOS_FILE;
use cartridge_input::{Action, Btn};
use cartridge_store::ini;
use cartridge_ui::Keyboard;

use crate::cheevos::Status;
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sel {
    Enabled,
    Account,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    User,
    Password,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountTyping {
    pub step: Step,
    /// The user name typed, once the password is being asked for.
    pub user: String,
    pub keys: Keyboard,
}

impl AccountTyping {
    pub fn prompt(&self) -> String {
        match self.step {
            Step::User => "RetroAchievements user name".to_string(),
            Step::Password => format!("RetroAchievements password for {}", self.user),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheevosScreen {
    pub sel: Sel,
    pub typing: Option<AccountTyping>,
}

impl Default for CheevosScreen {
    fn default() -> Self {
        CheevosScreen {
            sel: Sel::Enabled,
            typing: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Close,
    /// Switch RetroAchievements on or off.
    Toggle,
    /// An account was typed: its user name and password.
    SignIn,
    SignOut,
}

/// What the screen hands back with `Outcome::SignIn`.
pub type Typed = (String, String);

impl CheevosScreen {
    pub fn input(&mut self, action: Action, user: &str, typed: &mut Option<Typed>) -> Outcome {
        if self.typing.is_some() {
            return self.type_input(action, typed);
        }
        match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu => return Outcome::Close,
            Action::GbaDown(Btn::Up | Btn::Down) => {
                self.sel = match self.sel {
                    Sel::Enabled => Sel::Account,
                    Sel::Account => Sel::Enabled,
                }
            }
            Action::GbaDown(Btn::Left | Btn::Right) if self.sel == Sel::Enabled => {
                return Outcome::Toggle
            }
            Action::GbaDown(Btn::A) if self.sel == Sel::Enabled => return Outcome::Toggle,
            Action::GbaDown(Btn::A) if self.sel == Sel::Account => {
                self.typing = Some(AccountTyping {
                    step: Step::User,
                    user: String::new(),
                    keys: Keyboard {
                        text: user.to_string(),
                        ..Keyboard::default()
                    },
                });
            }
            Action::GbaDown(Btn::Y) if self.sel == Sel::Account => return Outcome::SignOut,
            _ => {}
        }
        Outcome::Stay
    }

    fn type_input(&mut self, action: Action, typed: &mut Option<Typed>) -> Outcome {
        let Some(t) = &mut self.typing else {
            return Outcome::Stay;
        };
        let keys = &mut t.keys;
        let mut submit = false;
        match action {
            Action::GbaDown(Btn::Up) => keys.up(),
            Action::GbaDown(Btn::Down) => keys.down(),
            Action::GbaDown(Btn::Left) => keys.left(),
            Action::GbaDown(Btn::Right) => keys.right(),
            Action::GbaDown(Btn::A) => submit = keys.press(),
            Action::GbaDown(Btn::B) if keys.text.is_empty() => {
                self.typing = None;
                return Outcome::Stay;
            }
            Action::GbaDown(Btn::B) => keys.delete(),
            Action::GbaDown(Btn::X) => keys.shift(),
            Action::GbaDown(Btn::Y) => keys.type_char(' '),
            Action::GbaDown(Btn::L1 | Btn::R1) => keys.symbols(),
            Action::GbaDown(Btn::Start) => submit = true,
            Action::QuickMenu => {
                self.typing = None;
                return Outcome::Stay;
            }
            _ => return Outcome::Stay,
        }
        if !submit || t.keys.text.is_empty() {
            return Outcome::Stay;
        }
        let text = std::mem::take(&mut t.keys.text);
        match t.step {
            Step::User => {
                t.step = Step::Password;
                t.user = text;
                t.keys = Keyboard::default();
                Outcome::Stay
            }
            Step::Password => {
                let user = std::mem::take(&mut t.user);
                self.typing = None;
                self.sel = Sel::Enabled;
                *typed = Some((user, text));
                Outcome::SignIn
            }
        }
    }

    /// The lines under the list, for how the sign-in stands. `online` is whether Wi-Fi has a
    /// network joined.
    pub fn status_lines(status: &Status, online: bool, on: bool) -> Vec<String> {
        let said = |a: &str, b: &str| vec![a.to_string(), b.to_string()];
        match status {
            Status::SignedOut => said(
                "Sign in with your RetroAchievements account.",
                "Softcore only: saves and states work.",
            ),
            Status::SigningIn if !online => said(
                "Join a Wi-Fi network to sign in.",
                "The sign-in goes on once it is joined.",
            ),
            Status::SigningIn => said("Signing in...", ""),
            Status::SignedIn(_) if !on => said("Achievements are off.", "Games are not followed."),
            Status::SignedIn(_) => said(
                "Games are followed for achievements.",
                "Unlocks show over the game.",
            ),
            Status::Rejected(reason) => said("Sign-in refused:", reason),
        }
        .into_iter()
        .filter(|l| !l.is_empty())
        .collect()
    }

    pub fn view(&self, status: &Status, online: bool, on: bool) -> ListView {
        let account = match status {
            Status::SignedIn(user) => RowValue::Name(user.clone()),
            Status::SigningIn => RowValue::Said("Signing in"),
            Status::Rejected(_) => RowValue::Said("Failed"),
            Status::SignedOut => RowValue::Said("Sign in"),
        };
        let mut legend = vec![LegendHint::Back];
        if self.sel == Sel::Account {
            legend.push(LegendHint::SignIn);
            if !matches!(status, Status::SignedOut) {
                legend.push(LegendHint::SignOut);
            }
        } else {
            legend.push(LegendHint::Change);
        }
        ListView {
            rows: vec![
                ViewRow {
                    label: RowLabel::Setting("Achievements"),
                    value: Some(RowValue::Setting(on)),
                    dim: false,
                },
                ViewRow {
                    label: RowLabel::Setting("Account"),
                    value: Some(account),
                    dim: false,
                },
            ],
            lit: Some(match self.sel {
                Sel::Enabled => 0,
                Sel::Account => 1,
            }),
            status: Self::status_lines(status, online, on),
            legend,
        }
    }
}

/// Keeps `user` and `password` for the next sign-in, in place of whatever was kept.
pub fn keep_account(root: &Path, user: &str, password: &str) -> std::io::Result<()> {
    ini::write(root, CHEEVOS_FILE, "username", user)?;
    ini::write(root, CHEEVOS_FILE, "password", password)?;
    ini::remove(root, CHEEVOS_FILE, "token")
}

/// Forgets the kept account.
pub fn forget_account(root: &Path) {
    let _ = std::fs::remove_file(root.join(CHEEVOS_FILE));
}
