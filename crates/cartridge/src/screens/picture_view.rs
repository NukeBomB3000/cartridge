//! The picture viewer, opened by A on a Polaroid: the folder's pictures one at a time over the
//! whole canvas. Left and Right (and L1 and R1) turn to the one before and after, Y fits the
//! picture to the screen or stretches it, A shows or hides its name and place (nothing is laid
//! over a picture until A asks), B closes.
//! Where the folder was left is kept, and a long A on the Polaroid starts again from the first.

use cartridge_input::{Action, Btn};
use cartridge_store::media::{self, Kind};
use cartridge_store::Platform;
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::kit::row::panel_height;
use cartridge_ui::{Draw, UndoFace};

use crate::app::{App, Phase};
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};
use crate::nav::Dir;
use crate::picture_loader::Loader;
use crate::screens::lone::{hold, rows_height, Lone};
use crate::video_mode::VideoMode;

/// How wide the panel is, in art pixels.
pub const WIDTH: u32 = 260;
/// The panel's title.
pub const TITLE: &str = "PICTURES";

/// The screen: the folder's pictures, the one showing, and whether A has asked for the panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewScreen {
    pub name: String,
    pub files: Vec<std::path::PathBuf>,
    pub index: usize,
    pub info: bool,
}

pub fn view_of(screen: &ViewScreen, stretched: bool) -> ListView {
    let file = screen
        .files
        .get(screen.index)
        .and_then(|f| f.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let row = |label, value: String| ViewRow {
        label: RowLabel::Setting(label),
        value: Some(RowValue::Picked(value)),
        dim: false,
    };
    ListView {
        rows: vec![
            row("Picture", file),
            row(
                "Number",
                format!("{} / {}", screen.index + 1, screen.files.len()),
            ),
            row(
                "Picture",
                if stretched { "STRETCH" } else { "FIT" }.to_string(),
            ),
        ],
        lit: None,
        status: Vec::new(),
        legend: vec![LegendHint::Back],
    }
}

pub fn viewer_face(view: &ListView, pal: &Palette) -> UndoFace {
    let content = rows_height(view.rows.len());
    let mut panel = panel_face(WIDTH, panel_height(content), TITLE, true, pal);
    hold(&mut panel, &view.rows, view.lit, pal);
    panel.face
}

impl App {
    /// Opens the viewer on the pictures of `folder`, at the one it was left at, or the first.
    pub(crate) fn show_pictures(&mut self, name: &str, folder: &std::path::Path, clean: bool) {
        let files = media::files(Kind::Pictures, folder);
        if files.is_empty() {
            return;
        }
        let room = (cartridge_gfx::out_w(), cartridge_gfx::out_h());
        let index = match &self.root {
            Some(root) if !clean => {
                crate::media_resume::read(root, &crate::media_resume::key(root, folder))
                    .map(|(i, _)| i)
                    .filter(|i| *i < files.len())
                    .unwrap_or(0)
            }
            _ => 0,
        };
        let loader = Loader::new(room);
        loader.load(files[index].clone());
        self.picture_loader = Some(loader);
        self.picture_folder = Some(folder.to_path_buf());
        self.set_platform(Platform::Video);
        if let Some(root) = self.root.clone() {
            self.video_mode = crate::video_mode::video_mode_for(&root, Platform::Video);
        }
        self.go(
            Phase::Pictures(ViewScreen {
                name: name.to_string(),
                files,
                index,
                info: false,
            }),
            Dir::Deeper,
        );
    }

    fn close_pictures(&mut self) {
        self.save_picture_place();
        self.picture_loader = None;
        self.picture_folder = None;
        self.go(Phase::Shelf, Dir::Back);
    }

    /// Keeps which picture of the folder is showing.
    pub(crate) fn save_picture_place(&mut self) {
        let (Some(root), Some(folder), Phase::Pictures(screen)) =
            (&self.root, &self.picture_folder, &self.phase)
        else {
            return;
        };
        crate::media_resume::write(
            root,
            &crate::media_resume::key(root, folder),
            screen.index,
            0,
        );
    }

    pub(crate) fn pictures_input(&mut self, action: Action) {
        let step = match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu | Action::Eject => {
                return self.close_pictures()
            }
            Action::GbaDown(Btn::A) => {
                if let Phase::Pictures(screen) = &mut self.phase {
                    screen.info = !screen.info;
                }
                0
            }
            Action::ShelfRight | Action::GbaDown(Btn::Right | Btn::R1) => 1i64,
            Action::ShelfLeft | Action::GbaDown(Btn::Left | Btn::L1) => -1,
            Action::PictureToggle | Action::GbaDown(Btn::Y) => {
                self.video_mode = match self.video_mode {
                    VideoMode::Stretch => VideoMode::Aspect,
                    _ => VideoMode::Stretch,
                };
                if let Some(root) = &self.root {
                    let _ =
                        crate::video_mode::write_video_mode(root, Platform::Video, self.video_mode);
                }
                0
            }
            _ => 0,
        };
        let Phase::Pictures(screen) = &mut self.phase else {
            return;
        };
        if step == 0 {
            return;
        }
        let n = screen.files.len() as i64;
        screen.index = (screen.index as i64 + step).rem_euclid(n) as usize;
        if let Some(loader) = &self.picture_loader {
            loader.load(screen.files[screen.index].clone());
        }
        self.save_picture_place();
    }

    /// The panel's list while it is up: only while A has asked for it, never by itself.
    pub fn viewer_view(&self) -> Option<ListView> {
        let Phase::Pictures(screen) = &self.phase else {
            return None;
        };
        screen
            .info
            .then(|| view_of(screen, self.video_mode == VideoMode::Stretch))
    }

    pub fn set_viewer_panel(&mut self, panel: Lone) {
        self.viewer_panel = Some(panel);
    }

    pub(crate) fn draw_viewer_screen(&self, out: &mut Vec<Draw>) {
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
        if self.viewer_view().is_some() {
            if let Some(panel) = &self.viewer_panel {
                panel.draw(out);
            }
        }
    }
}
