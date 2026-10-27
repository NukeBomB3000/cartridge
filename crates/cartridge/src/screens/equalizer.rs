//! The EQUALIZER screen, opened from Settings: a lone panel with the five bands as sliders
//! standing side by side, each its gain over it and its frequency under it, the preset and
//! the output they are for above them, and the keys under them.

use cartridge_input::Action;
use cartridge_store::{EQ_BANDS, EQ_DB_MAX};
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::pixel::{fill, text_width, LINE};
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::kit::text::text_face;
use cartridge_ui::{Draw, SettingsRow, UndoFace};

use crate::app::{App, Phase};
use crate::audio::EQ_HZ;
use crate::equalizer_screen::{band_name, preset_of, EqView, Outcome};
use crate::nav::Dir;
use crate::screens::lone::{notes, over, Lone, NOTE_GAP};

/// How wide the panel is, in art pixels.
pub const WIDTH: u32 = 220;

/// The panel's title.
pub const TITLE: &str = "EQUALIZER";

/// How tall a slider's track is: a pixel a dB either way, and the 0 dB line between.
const TRACK_H: u32 = 2 * EQ_DB_MAX as u32 + 1;

/// How many art pixels a dB moves the knob.
const DB_PX: u32 = 2;

/// How tall the sliders stand, their gain over them and their band under them.
const SLIDERS_H: u32 = LINE + 3 + TRACK_H * DB_PX + 3 + LINE;

/// The EQUALIZER panel showing `view`, in `pal`.
pub fn equalizer_face(view: &EqView, pal: &Palette) -> UndoFace {
    let head = LINE + NOTE_GAP;
    let keys = 2;
    let content = 2 + head + SLIDERS_H + NOTE_GAP + keys * LINE + 2;
    let mut panel = panel_face(WIDTH, panel_height(content), TITLE, true, pal);
    let output = if view.headphones {
        "HEADPHONES"
    } else {
        "SPEAKER"
    };
    let preset = preset_of(view.gains).unwrap_or("Custom").to_uppercase();
    notes(&mut panel, &[format!("{output}   PRESET {preset}")], 2, pal);

    let [wx, wy, ww, _] = panel.well;
    let top = wy + 2 + head;
    let col = ww / EQ_BANDS as u32;
    for (i, &db) in view.gains.iter().enumerate() {
        let lit = i == view.band;
        let x = wx + col * i as u32;
        let mid = x + col / 2;
        if lit {
            fill(
                &mut panel.face,
                [
                    x as i32 + 2,
                    top as i32 - 1,
                    col as i32 - 4,
                    SLIDERS_H as i32 + 2,
                ],
                pal.plate,
            );
        }
        let ink = if lit { pal.cream } else { pal.label };
        let gain = text_face(&format!("{db:+}"), ink);
        over(&mut panel.face, &gain, mid - gain.w / 2, top);

        // the track, the 0 dB line across it, and the bar from it to the knob
        let ty = top + LINE + 3;
        let th = TRACK_H * DB_PX;
        let zero = ty + EQ_DB_MAX as u32 * DB_PX;
        fill(
            &mut panel.face,
            [mid as i32 - 1, ty as i32, 2, th as i32],
            pal.edge,
        );
        fill(
            &mut panel.face,
            [mid as i32 - 5, zero as i32, 10, 1],
            pal.dim,
        );
        let knob = (zero as i32) - i32::from(db) * DB_PX as i32;
        let (from, to) = (knob.min(zero as i32), knob.max(zero as i32));
        let bar = if lit { pal.label } else { pal.frame };
        fill(
            &mut panel.face,
            [mid as i32 - 1, from, 2, to - from + 1],
            bar,
        );
        fill(
            &mut panel.face,
            [mid as i32 - 6, knob - 1, 12, 3],
            if lit { pal.cream } else { pal.frame_hi },
        );

        let name = text_face(&band_name(EQ_HZ[i]), if lit { pal.cream } else { pal.dim });
        over(&mut panel.face, &name, mid - name.w / 2, ty + th + 3);
    }

    let test = if view.playing { "STOP" } else { "PLAY" };
    let below = 2 + head + SLIDERS_H + NOTE_GAP;
    notes(
        &mut panel,
        &[
            "◀▶ BAND   ▲▼ GAIN".to_string(),
            format!("A {test} TEST SOUND   Y PRESET"),
        ],
        below,
        pal,
    );
    debug_assert!(text_width("A STOP TEST SOUND   Y PRESET") < ww);
    panel.face
}

impl App {
    /// The bands for what plays out now: the headphones' when they are in, else the speaker's.
    pub fn eq_gains(&self) -> [i8; EQ_BANDS] {
        if self.headphones {
            self.state.eq_hp
        } else {
            self.state.eq
        }
    }

    /// The test sound is asked for: the EQUALIZER screen is open and playing it.
    pub fn eq_test_playing(&self) -> bool {
        matches!(&self.phase, Phase::Equalizer(s) if s.playing)
    }

    pub(crate) fn equalizer_input(&mut self, action: Action) {
        let gains = self.eq_gains();
        let Phase::Equalizer(screen) = &mut self.phase else {
            return;
        };
        match screen.input(gains, action) {
            Outcome::Close => self.go(
                Phase::Settings {
                    row: SettingsRow::Equalizer,
                },
                Dir::Back,
            ),
            Outcome::Changed(next) => {
                if self.headphones {
                    self.state.eq_hp = next;
                } else {
                    self.state.eq = next;
                }
                self.persist();
            }
            Outcome::Stay => {}
        }
    }

    /// The EQUALIZER screen's view, while it is open.
    pub fn equalizer_view(&self) -> Option<EqView> {
        match &self.phase {
            Phase::Equalizer(screen) => Some(EqView {
                gains: self.eq_gains(),
                band: screen.band,
                playing: screen.playing,
                headphones: self.headphones,
            }),
            _ => None,
        }
    }

    pub fn set_equalizer_panel(&mut self, panel: Lone) {
        self.equalizer_panel = Some(panel);
    }

    pub(crate) fn draw_equalizer_screen(&self, out: &mut Vec<Draw>) {
        if let Some(panel) = &self.equalizer_panel {
            panel.draw(out);
        }
    }
}
