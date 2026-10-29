//! The music player's screen, opened by A on a tape of songs: a lone panel over the backdrop with
//! the song that plays, its place in the tape and how far into it. A pauses and plays, Left and
//! Right jump ten seconds, L1 and R1 go to the song before and after, B stops and puts the tape
//! back.

use cartridge_input::{Action, Btn};
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::{Draw, UndoFace};

use crate::app::{App, Phase};
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};
use crate::music_player::{clock, Cmd, Player, Status, SEEK_SECS};
use crate::nav::Dir;
use crate::screens::lone::{hold, notes, notes_height, rows_height, Lone, NOTE_GAP};

/// How long the player stands unused before the screen dims, in milliseconds.
pub const MUSIC_DIM_MS: cartridge_ui::Millis = 30_000;

/// How many times dimmer the screen is then: its brightness step divided by this.
const DIM_DIVISOR: u8 = 4;

/// How often the place in the tape is written, in milliseconds.
const SAVE_PLACE_MS: cartridge_ui::Millis = 5_000;

/// How wide the panel is, in art pixels.
pub const WIDTH: u32 = 300;

/// The panel's title.
pub const TITLE: &str = "NOW PLAYING";

/// The screen: the tape's name, which is printed on the panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MusicScreen {
    pub tape: String,
}

/// The list the screen shows for the song that plays.
pub fn view_of(tape: &str, status: &Status) -> ListView {
    let state = match (status.ended, status.paused) {
        (true, _) => "ENDED",
        (_, true) => "PAUSED",
        _ => "PLAYING",
    };
    let row = |label, value: String| ViewRow {
        label: RowLabel::Setting(label),
        value: Some(RowValue::Picked(value)),
        dim: false,
    };
    ListView {
        rows: vec![
            row("Song", status.title.clone()),
            row("Number", format!("{} / {}", status.number, status.count)),
            row(
                "Time",
                format!("{} / {}", clock(status.pos_ms), clock(status.dur_ms)),
            ),
            row("State", state.to_string()),
        ],
        lit: None,
        status: vec![tape.to_string()],
        legend: vec![LegendHint::Back],
    }
}

/// The panel holding `view`'s rows and note, in `pal`.
pub fn music_face(view: &ListView, pal: &Palette) -> UndoFace {
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
    /// The ring the handheld's sound goes out through, which the player feeds.
    pub fn set_audio_ring(&mut self, ring: std::sync::Arc<crate::audio::Ring>) {
        self.audio_ring = Some(ring);
    }

    /// Starts the songs of `folder` playing, under the name `tape`, and opens the screen.
    pub(crate) fn play_tape(&mut self, tape: &str, folder: &std::path::Path, clean: bool) {
        let Some(ring) = self.audio_ring.clone() else {
            return;
        };
        let songs = cartridge_store::music::songs(folder);
        if songs.is_empty() {
            return;
        }
        // from where the tape was left, unless a long A asked for the first song
        let key = self
            .root
            .as_ref()
            .map(|root| crate::media_resume::key(root, folder));
        let start = match (&self.root, &key) {
            (Some(root), Some(key)) if !clean => {
                crate::media_resume::read(root, key).unwrap_or((0, 0))
            }
            _ => (0, 0),
        };
        eprintln!(
            "cartridge: music: {} from song {} at {} ms",
            folder.display(),
            start.0 + 1,
            start.1
        );
        self.music_key = key;
        self.media_saved_at = self.now();
        self.music = Some(Player::start(ring, songs, self.output_volume(), start));
        self.go(
            Phase::Music(MusicScreen {
                tape: tape.to_string(),
            }),
            Dir::Deeper,
        );
    }

    /// Stops the music and puts the tape back on the shelf.
    fn stop_tape(&mut self) {
        self.save_music_place();
        self.music = None;
        self.go(Phase::Shelf, Dir::Back);
    }

    pub(crate) fn music_input(&mut self, action: Action) {
        let Some(player) = &self.music else {
            return self.stop_tape();
        };
        match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu => self.stop_tape(),
            Action::GbaDown(Btn::A) => player.send(Cmd::Toggle),
            Action::GbaDown(Btn::L1) => player.send(Cmd::Prev),
            Action::GbaDown(Btn::R1) => player.send(Cmd::Next),
            Action::ShelfLeft | Action::GbaDown(Btn::Left) => player.send(Cmd::Seek(-SEEK_SECS)),
            Action::ShelfRight | Action::GbaDown(Btn::Right) => player.send(Cmd::Seek(SEEK_SECS)),
            _ => {}
        }
    }

    /// The screen's list, while it is open.
    pub fn music_view(&self) -> Option<ListView> {
        let (Phase::Music(screen), Some(player)) = (&self.phase, &self.music) else {
            return None;
        };
        Some(view_of(&screen.tape, &player.status()))
    }

    pub fn set_music_panel(&mut self, panel: Lone) {
        self.music_panel = Some(panel);
    }

    pub(crate) fn draw_music_screen(&self, out: &mut Vec<Draw>) {
        if let Some(panel) = &self.music_panel {
            panel.draw(out);
        }
    }

    /// Writes the song the tape is at and how far into it, or forgets the place of a tape that
    /// has played out.
    pub(crate) fn save_music_place(&mut self) {
        let (Some(root), Some(key), Some(player)) = (&self.root, &self.music_key, &self.music)
        else {
            return;
        };
        let status = player.status();
        if status.ended {
            crate::media_resume::clear(root, key);
        } else {
            crate::media_resume::write(root, key, status.number - 1, status.pos_ms);
        }
    }

    /// Per tick: the song's volume follows the handheld's, and a tape that has played out goes
    /// back to the shelf.
    pub(crate) fn tick_music(&mut self) {
        let volume = self.output_volume();
        if let Some(player) = &self.music {
            player.set_volume(volume);
            if matches!(self.phase, Phase::Music(_)) && player.status().ended {
                self.stop_tape();
            }
        }
        // the player left unused for a while dims the screen and puts the stick lights out; a
        // press, or leaving the player, brings both back
        let idle = matches!(self.phase, Phase::Music(_))
            && self.now().saturating_sub(self.last_input_at) >= MUSIC_DIM_MS;
        if self.music.is_some() && self.now().saturating_sub(self.media_saved_at) >= SAVE_PLACE_MS {
            self.media_saved_at = self.now();
            self.save_music_place();
        }
        if idle != self.music_dim {
            self.music_dim = idle;
            self.dim_for_music(idle);
        }
    }

    /// Dims the screen to a fraction of what it is set to, and darkens the stick lights, or puts
    /// both back as they are set.
    fn dim_for_music(&mut self, dim: bool) {
        let step = if dim {
            (self.state.brightness / DIM_DIVISOR).max(1)
        } else {
            self.state.brightness
        };
        if let Some(power) = &mut self.power {
            power.set_backlight(step);
        }
        if dim {
            self.darken_sticks();
        } else {
            self.glow_sticks();
        }
    }
}
