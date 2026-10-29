//! The video player's screen, opened by A on a VHS tape: the picture over the whole canvas, and
//! over it, while the video is paused and for a few seconds after a press, a small panel with its
//! name and place. A pauses and plays, Left and Right jump ten seconds, Up and Down a minute, L1
//! and R1 go to the video before and after in its folder, X picks the next sound track, Y fits the
//! picture to the screen or stretches it over all of it, B stops.

use cartridge_input::{Action, Btn};
use cartridge_store::media::{self, Kind};
use cartridge_store::Platform;
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::{Draw, Millis, UndoFace};

use crate::app::{App, Phase};
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};
use crate::music_player::clock;
use crate::nav::Dir;
use crate::screens::lone::{hold, rows_height, Lone};
use crate::video_mode::VideoMode;
use crate::video_player::{Cmd, Player, Status, SEEK_BIG_SECS, SEEK_SECS};

/// How wide the panel is, in art pixels.
pub const WIDTH: u32 = 300;

/// The panel's title.
pub const TITLE: &str = "VIDEO";

/// How long the panel stays up after a press, in milliseconds.
pub const SHOWN_MS: Millis = 3500;

/// The screen: the tape's name and when the panel was last asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoScreen {
    pub tape: String,
    pub asked_at: Millis,
}

/// The list the panel shows for the video that plays.
pub fn view_of(status: &Status, stretched: bool) -> ListView {
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
    let mut rows = vec![
        row("Video", status.title.clone()),
        row(
            "Time",
            format!("{} / {}", clock(status.pos_ms), clock(status.dur_ms)),
        ),
        row("State", state.to_string()),
    ];
    if status.count > 1 {
        rows.insert(
            1,
            row("Number", format!("{} / {}", status.number, status.count)),
        );
    }
    if status.tracks > 1 {
        rows.push(row(
            "Sound",
            format!("{} / {}", status.track, status.tracks),
        ));
    }
    rows.push(row(
        "Picture",
        if stretched { "STRETCH" } else { "FIT" }.to_string(),
    ));
    ListView {
        rows,
        lit: None,
        status: Vec::new(),
        legend: vec![LegendHint::Back],
    }
}

/// The panel holding `view`'s rows, in `pal`.
pub fn video_face(view: &ListView, pal: &Palette) -> UndoFace {
    let content = rows_height(view.rows.len());
    let mut panel = panel_face(WIDTH, panel_height(content), TITLE, true, pal);
    hold(&mut panel, &view.rows, view.lit, pal);
    panel.face
}

impl App {
    /// The video player's program, on the card.
    fn ffmpeg_path(&self) -> Option<std::path::PathBuf> {
        let path = self.root.as_ref()?.join("System/ffmpeg");
        path.is_file().then_some(path)
    }

    /// The videos played from `tape` on: a video's file with the rest of its folder after it, or
    /// the one video of a folder.
    fn playlist(tape: &std::path::Path) -> Vec<std::path::PathBuf> {
        if tape.is_file() {
            let all = tape
                .parent()
                .map(|dir| media::files(Kind::Video, dir))
                .unwrap_or_default();
            return match all.iter().position(|p| p == tape) {
                Some(at) => all[at..].to_vec(),
                None => vec![tape.to_path_buf()],
            };
        }
        media::files(Kind::Video, tape)
    }

    /// Starts the video of `tape` playing and opens its screen.
    pub(crate) fn play_video(&mut self, name: &str, tape: &std::path::Path, clean: bool) {
        let (Some(ring), Some(ffmpeg)) = (self.audio_ring.clone(), self.ffmpeg_path()) else {
            eprintln!("cartridge: video: no System/ffmpeg on the card");
            return;
        };
        let files = Self::playlist(tape);
        if files.is_empty() {
            return;
        }
        let room = (cartridge_gfx::out_w(), cartridge_gfx::out_h());
        // from where the video was left, unless a long A asked for the beginning
        let kept = match (&self.root, files.first()) {
            (Some(root), Some(first)) => {
                let key = crate::media_resume::key(root, first);
                if clean {
                    crate::media_resume::clear(root, &key);
                    None
                } else {
                    crate::media_resume::read(root, &key).map(|(_, ms)| ms)
                }
            }
            _ => None,
        };
        eprintln!(
            "cartridge: video: {} from {} ms",
            files[0].display(),
            kept.unwrap_or(0)
        );
        self.media_saved_at = self.now();
        self.video = Some(Player::start(
            ffmpeg,
            ring,
            files,
            room,
            self.output_volume(),
            kept,
        ));
        self.set_platform(Platform::Video);
        if let Some(root) = self.root.clone() {
            self.video_mode = crate::video_mode::video_mode_for(&root, Platform::Video);
        }
        let asked_at = self.now();
        self.go(
            Phase::Video(VideoScreen {
                tape: name.to_string(),
                asked_at,
            }),
            Dir::Deeper,
        );
    }

    fn stop_video(&mut self) {
        self.save_video_place();
        self.video = None;
        self.go(Phase::Shelf, Dir::Back);
    }

    pub(crate) fn video_input(&mut self, action: Action) {
        let now = self.now();
        if let Phase::Video(screen) = &mut self.phase {
            screen.asked_at = now;
        }
        let Some(player) = &self.video else {
            return self.stop_video();
        };
        match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu | Action::Eject => self.stop_video(),
            Action::GbaDown(Btn::A) => player.send(Cmd::Toggle),
            Action::GbaDown(Btn::L1) => player.send(Cmd::Prev),
            Action::GbaDown(Btn::R1) => player.send(Cmd::Next),
            Action::GbaDown(Btn::X) => player.send(Cmd::Track),
            Action::ShelfLeft | Action::GbaDown(Btn::Left) => player.send(Cmd::Seek(-SEEK_SECS)),
            Action::ShelfRight | Action::GbaDown(Btn::Right) => player.send(Cmd::Seek(SEEK_SECS)),
            Action::GbaDown(Btn::Up) => player.send(Cmd::Seek(SEEK_BIG_SECS)),
            Action::GbaDown(Btn::Down) => player.send(Cmd::Seek(-SEEK_BIG_SECS)),
            Action::PictureToggle | Action::GbaDown(Btn::Y) => {
                self.video_mode = match self.video_mode {
                    VideoMode::Stretch => VideoMode::Aspect,
                    _ => VideoMode::Stretch,
                };
                if let Some(root) = &self.root {
                    let _ =
                        crate::video_mode::write_video_mode(root, Platform::Video, self.video_mode);
                }
            }
            _ => {}
        }
    }

    /// The panel's list while it is up: the video is paused, or something was pressed lately.
    pub fn video_view(&self) -> Option<ListView> {
        let (Phase::Video(screen), Some(player)) = (&self.phase, &self.video) else {
            return None;
        };
        let status = player.status();
        let shown = status.paused || self.now().saturating_sub(screen.asked_at) < SHOWN_MS;
        shown.then(|| view_of(&status, self.video_mode == VideoMode::Stretch))
    }

    pub fn set_video_panel(&mut self, panel: Lone) {
        self.video_panel = Some(panel);
    }

    /// The newest picture of the video that plays.
    pub fn video_frame(&self) -> Option<crate::frames::FrameRef> {
        self.video
            .as_ref()
            .and_then(|p| p.frame())
            .or_else(|| self.picture_loader.as_ref().and_then(|l| l.frame()))
    }

    /// A picture of the video has been shown.
    pub fn video_published(&self) -> bool {
        self.video.as_ref().is_some_and(|p| p.published())
            || self.picture_loader.as_ref().is_some_and(|l| l.published())
    }

    pub(crate) fn draw_video_screen(&self, out: &mut Vec<Draw>) {
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: cartridge_gfx::out_w() as f32,
            h: cartridge_gfx::out_h() as f32,
            colour: [0.0, 0.0, 0.0, 1.0],
        });
        if self.game_ready {
            out.push(Draw::Game);
        }
        if self.video_view().is_some() {
            if let Some(panel) = &self.video_panel {
                panel.draw(out);
            }
        }
    }

    /// Writes how far into its file the video is.
    pub(crate) fn save_video_place(&mut self) {
        let (Some(root), Some(player)) = (&self.root, &self.video) else {
            return;
        };
        let status = player.status();
        // nothing is written over what was kept before a picture of the video has been shown
        if status.file.as_os_str().is_empty() || !player.published() {
            return;
        }
        let key = crate::media_resume::key(root, &status.file);
        if status.ended {
            crate::media_resume::clear(root, &key);
        } else {
            crate::media_resume::write(root, &key, 0, status.pos_ms);
        }
    }

    /// Per tick: the video's volume follows the handheld's, and a video that has played out
    /// goes back to the shelf.
    pub(crate) fn tick_video(&mut self) {
        let volume = self.output_volume();
        let Some(player) = &self.video else {
            return;
        };
        player.set_volume(volume);
        if matches!(self.phase, Phase::Video(_)) && player.status().ended {
            self.stop_video();
            return;
        }
        if self.now().saturating_sub(self.media_saved_at) >= 5_000 {
            self.media_saved_at = self.now();
            self.save_video_place();
        }
    }
}
