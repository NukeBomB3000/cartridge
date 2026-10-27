//! The cart's colour, Y on the shelf: a small panel at the foot of the shelf, the cart above it
//! taking each change as it is made. Its COLOUR row steps through pigments carts came in, after
//! slot-fragile's, its FINISH row through solid, clear and glitter plastic; each change is kept
//! in Config/cart_shell.ini, where the shelf reads a cart's shell from. ORIGINAL takes the cart's
//! own colour back. B or Y close it. A disc has no shell to colour.

use cartridge_input::{Action, Btn};
use cartridge_store::{ini, Outline, Platform, ShellChoice, ShellFinish, CART_SHELL_FILE};
use cartridge_ui::kit::layout::{page, screen_px};
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::pixel::quad;
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::{Draw, UndoFace};

use crate::app::App;
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};
use crate::screens::lone::{hold, notes, notes_height, rows_height, Lone, NOTE_GAP};

/// The colours a cart can be made in, the first its own.
pub const PIGMENTS: [(&str, Option<[u8; 3]>); 12] = [
    ("Original", None),
    ("Neutral Grey", Some([0x9a, 0x9a, 0xa0])),
    ("Ruby Red", Some([0xb0, 0x1c, 0x2c])),
    ("Sapphire Blue", Some([0x22, 0x3c, 0x9c])),
    ("Emerald Green", Some([0x1e, 0x8a, 0x4e])),
    ("Gold", Some([0xc8, 0xa2, 0x3c])),
    ("FireRed", Some([0xd8, 0x4a, 0x1c])),
    ("LeafGreen", Some([0x6a, 0xb8, 0x3a])),
    ("Jet Black", Some([0x1c, 0x1c, 0x20])),
    ("Atomic Purple", Some([0x6a, 0x4a, 0xa8])),
    ("White", Some([0xe8, 0xe8, 0xe8])),
    ("Pink", Some([0xe0, 0x6a, 0xa0])),
];

const FINISHES: [ShellFinish; 3] = [ShellFinish::Solid, ShellFinish::Clear, ShellFinish::Glitter];

fn finish_name(finish: ShellFinish) -> &'static str {
    match finish {
        ShellFinish::Solid => "Solid",
        ShellFinish::Clear => "Clear",
        ShellFinish::Glitter => "Glitter",
    }
}

/// How wide the panel is, in art pixels.
pub const WIDTH: u32 = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellScreen {
    /// 0 the colour row, 1 the finish row.
    pub row: usize,
    /// The pigment picked, in [`PIGMENTS`].
    pub colour: usize,
    pub finish: ShellFinish,
    /// The outline the cart's choice kept, which this screen leaves as it was.
    pub outline: Outline,
}

impl ShellScreen {
    /// The screen for a cart wearing `choice`, if any: its colour found among the pigments, or
    /// its own.
    pub fn of(choice: Option<ShellChoice>) -> ShellScreen {
        let colour = choice
            .and_then(|c| PIGMENTS.iter().position(|(_, rgb)| *rgb == Some(c.colour)))
            .unwrap_or(0);
        ShellScreen {
            row: 0,
            colour,
            finish: choice.map_or(ShellFinish::Solid, |c| c.finish),
            outline: choice.map_or(Outline::Auto, |c| c.outline),
        }
    }

    /// The choice the screen stands at: none for the cart's own colour, solid.
    pub fn choice(&self) -> Option<ShellChoice> {
        let rgb = PIGMENTS[self.colour].1;
        if rgb.is_none() && self.finish == ShellFinish::Solid {
            return None;
        }
        Some(ShellChoice {
            outline: self.outline,
            // the cart's own colour, clear or glittered: grey stands in for it
            colour: rgb.unwrap_or([0x9a, 0x9a, 0xa0]),
            finish: self.finish,
        })
    }

    /// A press: true when it changed the cart.
    pub fn input(&mut self, action: Action) -> bool {
        let step = |at: usize, n: usize, forward: bool| {
            if forward {
                (at + 1) % n
            } else {
                (at + n - 1) % n
            }
        };
        match action {
            Action::GbaDown(Btn::Up | Btn::Down) => {
                self.row = 1 - self.row;
                false
            }
            Action::GbaDown(b @ (Btn::Left | Btn::Right)) => {
                let forward = b == Btn::Right;
                if self.row == 0 {
                    self.colour = step(self.colour, PIGMENTS.len(), forward);
                } else {
                    let at = FINISHES.iter().position(|f| *f == self.finish).unwrap_or(0);
                    self.finish = FINISHES[step(at, FINISHES.len(), forward)];
                }
                true
            }
            _ => false,
        }
    }

    pub fn view(&self) -> ListView {
        ListView {
            rows: vec![
                ViewRow {
                    label: RowLabel::Setting("Colour"),
                    value: Some(RowValue::Choice(PIGMENTS[self.colour].0)),
                    dim: false,
                },
                ViewRow {
                    label: RowLabel::Setting("Finish"),
                    value: Some(RowValue::Choice(finish_name(self.finish))),
                    dim: false,
                },
            ],
            lit: Some(self.row),
            status: vec!["◀▶ CHANGE   B DONE".into()],
            legend: vec![LegendHint::Back, LegendHint::Change],
        }
    }
}

/// The panel holding `view`'s rows and its note, in `pal`.
pub fn shell_face(view: &ListView, pal: &Palette) -> UndoFace {
    let content = rows_height(view.rows.len()) + notes_height(view.status.len());
    let mut panel = panel_face(WIDTH, panel_height(content), "CART", true, pal);
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
    /// Y on the shelf: the cart's colour, for any cart but a disc.
    pub(crate) fn open_shell_screen(&mut self) {
        let Some(cart) = self.selected_cart() else {
            return;
        };
        if cart.platform == Platform::Ps1 {
            return self.refuse();
        }
        self.shell_screen = Some(ShellScreen::of(cart.shell));
    }

    pub(crate) fn shell_input(&mut self, action: Action) {
        let Some(screen) = self.shell_screen.as_mut() else {
            return;
        };
        if matches!(action, Action::GbaDown(Btn::B | Btn::Y)) {
            self.shell_screen = None;
            return;
        }
        if !screen.input(action) {
            return;
        }
        let choice = screen.choice();
        let Some((platform, stem)) = self.selected_cart().map(|c| (c.platform, c.stem.clone()))
        else {
            return;
        };
        if let Some(root) = self.root.as_deref() {
            let key = cartridge_store::cart_shell_key(&stem);
            let written = match choice {
                Some(c) => ini::write(root, CART_SHELL_FILE, &key, &c.to_value()),
                None => ini::remove(root, CART_SHELL_FILE, &key),
            };
            if let Err(e) = written {
                eprintln!("cartridge: {CART_SHELL_FILE}: {e}");
            }
        }
        let mine = |c: &&mut cartridge_store::Cart| c.platform == platform && c.stem == stem;
        for cart in self
            .shelves
            .iter_mut()
            .flat_map(|(_, s)| s.carts.iter_mut())
            .chain(self.all_carts.iter_mut())
            .filter(|c| mine(c))
        {
            cart.shell = choice;
        }
        self.stale_faces.push((platform, stem));
    }

    /// The panel's list, while it is open.
    pub fn shell_view(&self) -> Option<ListView> {
        self.shell_screen.as_ref().map(ShellScreen::view)
    }

    pub fn set_shell_panel(&mut self, panel: Lone) {
        self.shell_panel = Some(panel);
    }

    /// The panel at the foot of the shelf, under the cart it colours.
    pub(crate) fn draw_shell_screen(&self, out: &mut Vec<Draw>) {
        if self.shell_screen.is_none() {
            return;
        }
        let Some(panel) = &self.shell_panel else {
            return;
        };
        let [pw, ph] = page();
        let x = pw.saturating_sub(panel.w) / 2;
        let y = ph.saturating_sub(panel.h + 2);
        out.push(quad(
            panel.tex,
            panel.w,
            panel.h,
            screen_px(x),
            screen_px(y),
        ));
    }
}
