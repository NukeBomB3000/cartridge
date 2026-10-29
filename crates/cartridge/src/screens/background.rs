//! The THEME screen, opened from Settings: a lone panel, as flow.html's `#screen=background`,
//! its rows BACKGROUND, MOTION and COLOUR drawn over the background they are choosing, and FONT
//! over the bar it is choosing for. A new colour re-tints the panel, and every screen after it,
//! the next frame; a new font redraws the bar's actions under it in that frame too.

use cartridge_input::Action;
use cartridge_store::StickLights;
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::{Draw, SettingsRow, UndoFace};

use crate::app::{App, Phase};
use crate::background_screen::Outcome as BackgroundOutcome;
use crate::list_view::ListView;
use crate::nav::Dir;
use crate::screens::lone::{hold, rows_height, Lone};

/// How wide the panel is, in art pixels: flow.css's .lone.small.
pub const WIDTH: u32 = 200;

/// The panel's title.
pub const TITLE: &str = "THEME";

/// The THEME panel holding `view`'s rows, in `pal`.
pub fn background_face(view: &ListView, pal: &Palette) -> UndoFace {
    let content = rows_height(view.rows.len());
    let mut panel = panel_face(WIDTH, panel_height(content), TITLE, true, pal);
    hold(&mut panel, &view.rows, view.lit, pal);
    panel.face
}

impl App {
    pub(crate) fn background_input(&mut self, action: Action) {
        let (look, font, lights, wallpaper) = (
            self.backdrop(),
            self.state.font,
            self.has_stick_lights().then_some(self.state.stick_lights),
            self.wallpaper.is_some(),
        );
        let Phase::Background(screen) = &mut self.phase else {
            return;
        };
        match screen.input(look, font, lights, wallpaper, action) {
            BackgroundOutcome::Close => self.go(
                Phase::Settings {
                    row: SettingsRow::Background,
                },
                Dir::Back,
            ),
            BackgroundOutcome::Changed(b) => {
                self.state.backdrop = Some(b.style);
                self.state.backdrop_moving = b.moving;
                self.state.backdrop_colour = b.colour;
                self.persist();
                self.glow_sticks();
            }
            BackgroundOutcome::Font(font) => {
                self.state.font = font;
                self.persist();
            }
            BackgroundOutcome::Lights(lights) => {
                self.state.stick_lights = lights;
                self.persist();
                self.glow_sticks();
            }
            BackgroundOutcome::Stay => {}
        }
    }

    /// The THEME screen's list, while it is open.
    pub fn background_view(&self) -> Option<ListView> {
        match &self.phase {
            Phase::Background(screen) => Some(screen.view(
                self.backdrop(),
                self.state.font,
                self.has_stick_lights().then_some(self.state.stick_lights),
            )),
            _ => None,
        }
    }

    /// The handheld has lights in its sticks: the card carries the script that drives them, and
    /// BaseOS says it is a model that has them. Anywhere else the row is left out.
    pub fn has_stick_lights(&self) -> bool {
        let Some(root) = &self.root else {
            return false;
        };
        root.join("System/rgb.sh").exists()
            && std::fs::read_to_string("/etc/baseos-release").is_ok_and(|text| {
                ["rg40xxh", "rg40xxv", "rgcubexx"]
                    .iter()
                    .any(|t| text.contains(&format!("BASEOS_TARGET={t}")))
            })
    }

    /// Sets the sticks' lights to what the card remembers: `System/rgb.sh` does it, on a
    /// thread of its own as it waits on the lights' controller. A card without the script, and
    /// a handheld without lights, do nothing.
    pub(crate) fn glow_sticks(&self) {
        let Some(root) = &self.root else {
            return;
        };
        if !self.has_stick_lights() {
            return;
        }
        let script = root.join("System/rgb.sh");
        let [r, g, b] = cartridge_ui::backdrop_rgb(self.state.backdrop_colour);
        let [r, g, b] = [r.to_string(), g.to_string(), b.to_string()];
        let mut cmd = std::process::Command::new("/bin/sh");
        cmd.arg(script);
        match self.state.stick_lights {
            StickLights::Off => cmd.arg("off"),
            StickLights::Colour => cmd.args(["solid", &r, &g, &b, "40"]),
            StickLights::Breathe => cmd.args(["breath", "slow", &r, &g, &b, "40"]),
            StickLights::Rainbow => cmd.args(["rainbow", "40"]),
        };
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        std::thread::spawn(move || {
            let _ = cmd.status();
        });
    }

    pub fn set_background_panel(&mut self, panel: Lone) {
        self.background_panel = Some(panel);
    }

    /// The panel, over the background drawn under every screen, with nothing between.
    pub(crate) fn draw_background_screen(&self, out: &mut Vec<Draw>) {
        if let Some(panel) = &self.background_panel {
            panel.draw(out);
        }
    }
}
