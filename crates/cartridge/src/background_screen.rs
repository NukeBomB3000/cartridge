//! The THEME screen, opened from Settings: what the shelf is drawn over, shown behind the list
//! as it is changed, and the font the bar's actions write their words in.

use cartridge_input::{Action, Btn};
use cartridge_store::{
    BackdropStyle, Font, StickLights, BACKDROP_COLOURS, LIGHT_COLOURS, LIGHT_LEVEL_MAX,
};
use cartridge_ui::backdrop_colour_name;

use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};

/// The shelf's background as it is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backdrop {
    pub style: BackdropStyle,
    pub moving: bool,
    pub colour: u8,
}

impl Backdrop {
    /// What the settings come to on a card that does or does not have a wallpaper: one that
    /// never picked shows the wallpaper if there is one, as cartridge always did, else the grid.
    pub fn resolve(
        picked: Option<BackdropStyle>,
        moving: bool,
        colour: u8,
        wallpaper: bool,
    ) -> Self {
        let style = match picked {
            Some(BackdropStyle::Wallpaper) | None if wallpaper => BackdropStyle::Wallpaper,
            Some(BackdropStyle::Wallpaper) | None => BackdropStyle::Grid,
            Some(style) => style,
        };
        Backdrop {
            style,
            moving,
            colour: colour.min(BACKDROP_COLOURS.len() as u8 - 1),
        }
    }
}

/// How the sticks' lights are set, on a handheld that has them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Glow {
    pub mode: StickLights,
    pub colour: u8,
    pub level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sel {
    /// The background's style, on the row named Background.
    Style,
    Motion,
    Colour,
    Font,
    /// The analog sticks' lights, on a handheld that has them.
    Lights,
    /// Their own colour, when they are not in the theme's.
    LightColour,
    /// Their own brightness, when they are not following the display's.
    LightLevel,
}

impl Sel {
    /// The panel's rows, top to bottom.
    pub const ALL: [Sel; 7] = [
        Sel::Style,
        Sel::Motion,
        Sel::Colour,
        Sel::Font,
        Sel::Lights,
        Sel::LightColour,
        Sel::LightLevel,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundScreen {
    pub sel: Sel,
}

impl Default for BackgroundScreen {
    fn default() -> Self {
        BackgroundScreen { sel: Sel::Style }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Close,
    Changed(Backdrop),
    Font(Font),
    Lights(Glow),
}

/// The styles a card offers: the wallpaper only where there is one.
fn styles(wallpaper: bool) -> Vec<BackdropStyle> {
    BackdropStyle::ALL
        .into_iter()
        .filter(|s| *s != BackdropStyle::Wallpaper || wallpaper)
        .collect()
}

fn step<T: PartialEq + Copy>(items: &[T], at: T, forward: bool) -> T {
    let n = items.len();
    let i = items.iter().position(|x| *x == at).unwrap_or(0);
    items[if forward {
        (i + 1) % n
    } else {
        (i + n - 1) % n
    }]
}

impl BackgroundScreen {
    /// Whether a style has a use for a row: a picture has no colour, a plain colour does not
    /// move. A row it has none for stays on the panel, faint, so the panel keeps its size.
    /// Every style has a use for the font, which is the bar's.
    fn applies(look: Backdrop, sel: Sel) -> bool {
        match sel {
            Sel::Style | Sel::Font | Sel::Lights => true,
            // only a colour of their own has a colour and a brightness to pick
            Sel::LightColour | Sel::LightLevel => true,
            Sel::Motion => look.style.moves(),
            Sel::Colour => look.style.coloured(),
        }
    }

    /// Whether the panel has a row at all: the lights' only where there are lights.
    fn shown(sel: Sel, lights: Option<Glow>) -> bool {
        match sel {
            Sel::Lights | Sel::LightColour | Sel::LightLevel => lights.is_some(),
            _ => true,
        }
    }

    /// Whether a lights row has a use: a colour and a brightness of their own only in the
    /// Colour mode, which leaves them faint in the others.
    fn lit(sel: Sel, lights: Option<Glow>) -> bool {
        match sel {
            Sel::LightColour | Sel::LightLevel => {
                lights.is_some_and(|g| g.mode == StickLights::Colour)
            }
            _ => true,
        }
    }

    /// The rows the cursor can stand on for this style.
    fn rows(look: Backdrop, lights: Option<Glow>) -> Vec<Sel> {
        Sel::ALL
            .into_iter()
            .filter(|sel| Self::applies(look, *sel) && Self::shown(*sel, lights))
            .filter(|sel| Self::lit(*sel, lights))
            .collect()
    }

    pub fn input(
        &mut self,
        look: Backdrop,
        font: Font,
        lights: Option<Glow>,
        wallpaper: bool,
        action: Action,
    ) -> Outcome {
        let rows = Self::rows(look, lights);
        if !rows.contains(&self.sel) {
            self.sel = Sel::Style;
        }
        let forward = match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu => return Outcome::Close,
            Action::GbaDown(Btn::Up) => {
                self.sel = step(&rows, self.sel, false);
                return Outcome::Stay;
            }
            Action::GbaDown(Btn::Down) => {
                self.sel = step(&rows, self.sel, true);
                return Outcome::Stay;
            }
            Action::GbaDown(Btn::Left) => false,
            Action::GbaDown(Btn::Right | Btn::A) => true,
            _ => return Outcome::Stay,
        };
        let mut next = look;
        match self.sel {
            Sel::Style => next.style = step(&styles(wallpaper), look.style, forward),
            Sel::Motion => next.moving = !look.moving,
            Sel::Colour => {
                let colours: Vec<u8> = (0..BACKDROP_COLOURS.len() as u8).collect();
                next.colour = step(&colours, look.colour, forward);
            }
            Sel::Font => return Outcome::Font(step(&Font::ALL, font, forward)),
            Sel::Lights | Sel::LightColour | Sel::LightLevel => {
                let mut glow = lights.unwrap_or_default();
                match self.sel {
                    Sel::Lights => glow.mode = step(&StickLights::ALL, glow.mode, forward),
                    Sel::LightColour => {
                        let colours: Vec<u8> = (0..LIGHT_COLOURS.len() as u8).collect();
                        glow.colour = step(&colours, glow.colour, forward);
                    }
                    _ => {
                        glow.level = if forward {
                            (glow.level + 1).min(LIGHT_LEVEL_MAX)
                        } else {
                            glow.level.saturating_sub(1).max(1)
                        }
                    }
                }
                return Outcome::Lights(glow);
            }
        }
        Outcome::Changed(next)
    }

    pub fn view(&self, look: Backdrop, font: Font, lights: Option<Glow>) -> ListView {
        let shown: Vec<Sel> = Sel::ALL
            .into_iter()
            .filter(|sel| Self::shown(*sel, lights))
            .collect();
        let at = shown.iter().position(|r| *r == self.sel).unwrap_or(0);
        let row = |sel: &Sel| match sel {
            Sel::Style => ViewRow {
                label: RowLabel::Setting("Background"),
                value: Some(RowValue::Choice(look.style.label())),
                dim: false,
            },
            Sel::Motion => ViewRow {
                label: RowLabel::Setting("Motion"),
                value: Some(RowValue::Choice(if look.moving {
                    "Moving"
                } else {
                    "Still"
                })),
                dim: !Self::applies(look, Sel::Motion),
            },
            Sel::Colour => ViewRow {
                label: RowLabel::Setting("Colour"),
                value: Some(RowValue::Choice(backdrop_colour_name(look.colour))),
                dim: !Self::applies(look, Sel::Colour),
            },
            Sel::Font => ViewRow {
                label: RowLabel::Setting("Font"),
                value: Some(RowValue::Choice(font.label())),
                dim: false,
            },
            Sel::Lights => ViewRow {
                label: RowLabel::Setting("Stick Lights"),
                value: Some(RowValue::Choice(lights.unwrap_or_default().mode.label())),
                dim: false,
            },
            Sel::LightColour => ViewRow {
                label: RowLabel::Setting("Light Colour"),
                value: Some(RowValue::Choice(
                    LIGHT_COLOURS
                        [(lights.unwrap_or_default().colour as usize).min(LIGHT_COLOURS.len() - 1)]
                    .1,
                )),
                dim: !Self::lit(Sel::LightColour, lights),
            },
            Sel::LightLevel => ViewRow {
                label: RowLabel::Setting("Light Brightness"),
                value: Some(RowValue::Picked(format!(
                    "{}0%",
                    lights.unwrap_or_default().level
                ))),
                dim: !Self::lit(Sel::LightLevel, lights),
            },
        };
        // the panel holds its rows and nothing under them, as flow.html draws it
        ListView {
            rows: shown.iter().map(row).collect(),
            lit: Some(at),
            status: Vec::new(),
            legend: vec![LegendHint::Back, LegendHint::Change],
        }
    }
}
