//! The game guide a short MENU press opens over a game: a walkthrough the player keeps in
//! GUIDES/<system>/ named as the game is, TXT or HTML, read over the paused game. L1 and R1
//! (or Left and Right) turn a page, Up and Down move a line and keep moving while held, L2 and
//! R2 make the text smaller or larger, X goes on to the game's next guide when it has several
//! (kept in a folder named as the game), and B or MENU close it. A second MENU press straight
//! after the first is still a double press and opens the save states.
//!
//! Where the player was in each guide, and the size they read at, are kept in
//! Config/guides.txt, so the guide opens where it was left, at any size.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use cartridge_gfx::{out_h, out_w, Draw, TexId};
use cartridge_input::{Action, Btn};
use cartridge_ui::guide::{
    decode, is_guide_file, lay_out, max_top, parse, render_page, text_area, Advances, Block,
    Glyphs, GuideInk, Laid, Page, GUIDE_SIZES, GUIDE_SIZE_DEFAULT,
};
use cartridge_ui::{clean_label, Toast};

use crate::app::App;

/// The folder on the card the guides are kept in, a folder for each system under it.
pub const GUIDES_DIR: &str = "GUIDES";
const MARKS_FILE: &str = "guides.txt";

/// How long Up or Down is held before the guide keeps moving, and how often it moves then.
const REPEAT_AFTER_MS: f64 = 350.0;
const REPEAT_EVERY_MS: f64 = 55.0;

/// How long after the place in a guide last moved it is written to the card.
const MARK_AFTER_MS: f64 = 2000.0;

pub const KEYS: &str = "L/R PAGE   ▲▼ SCROLL   L2/R2 SIZE   B BACK";
pub const KEYS_MORE: &str = "L/R PAGE   ▲▼ SCROLL   L2/R2 SIZE   X NEXT GUIDE   B BACK";

/// Where the player was in each guide, by its path under GUIDES, and the size they read at.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuideMarks {
    pub size: Option<usize>,
    pub at: HashMap<String, usize>,
}

impl GuideMarks {
    pub fn parse(text: &str) -> Self {
        let mut marks = GuideMarks::default();
        for line in text.lines() {
            if let Some(size) = line.strip_prefix("size=") {
                marks.size = size.trim().parse().ok().filter(|&s| s < GUIDE_SIZES.len());
            } else if let Some((at, key)) = line.split_once('\t') {
                if let Ok(at) = at.trim().parse() {
                    marks.at.insert(key.to_string(), at);
                }
            }
        }
        marks
    }

    pub fn print(&self) -> String {
        let mut out = format!("size={}\n", self.size.unwrap_or(GUIDE_SIZE_DEFAULT));
        let mut keys: Vec<_> = self.at.iter().collect();
        keys.sort();
        for (key, at) in keys {
            out.push_str(&format!("{at}\t{key}\n"));
        }
        out
    }

    fn load(root: &Path) -> Self {
        std::fs::read_to_string(marks_path(root))
            .map(|t| Self::parse(&t))
            .unwrap_or_default()
    }

    fn store(&self, root: &Path) {
        let path = marks_path(root);
        let tmp = path.with_extension("txt.tmp");
        let wrote = std::fs::write(&tmp, self.print()).and_then(|_| std::fs::rename(&tmp, &path));
        if let Err(e) = wrote {
            eprintln!("cartridge: guide marks: {e}");
        }
    }
}

fn marks_path(root: &Path) -> PathBuf {
    root.join(cartridge_store::CONFIG_DIR).join(MARKS_FILE)
}

/// A name as the guides are matched by: its letters and figures, lower case, with what is in
/// brackets left out, so "Golden Sun (Germany)" finds "golden sun.txt".
pub fn match_name(name: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in name.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            c if depth == 0 && c.is_alphanumeric() => out.extend(c.to_lowercase()),
            _ => {}
        }
    }
    out
}

/// The guides kept for the game `stem` in `dir`: a file named as the game, or every guide in
/// a folder named as it, in order.
pub fn find_guides(dir: &Path, stem: &str) -> Vec<PathBuf> {
    let want = match_name(stem);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    let mut folders = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if match_name(&name) == want {
                folders.push(path);
            }
        } else if is_guide_file(&name) {
            let stem_of = Path::new(&name)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if match_name(&stem_of) == want {
                files.push(path);
            }
        }
    }
    for folder in folders {
        if let Ok(entries) = std::fs::read_dir(folder) {
            files.extend(
                entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_file() && is_guide_file(&p.to_string_lossy())),
            );
        }
    }
    files.sort();
    files
}

/// The guide while it is open.
pub struct GuideScreen {
    pub guides: Vec<PathBuf>,
    pub index: usize,
    pub title: String,
    blocks: Vec<Block>,
    adv: Advances,
    glyphs: Glyphs,
    pub laid: Laid,
    pub top: f32,
    pub size: usize,
    /// Up or Down held, and when the guide next moves for it.
    held: Option<(Btn, f64)>,
    /// Whether the page has changed since it was last drawn.
    pub stale: bool,
    /// When the place last moved, until it is written to the card.
    moved: Option<f64>,
}

impl GuideScreen {
    fn load(path: &Path) -> Option<(String, Vec<Block>)> {
        let bytes = std::fs::read(path).ok()?;
        let text = decode(&bytes);
        let lower = path.to_string_lossy().to_ascii_lowercase();
        let blocks = parse(&text, lower.ends_with(".html") || lower.ends_with(".htm"));
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        Some((title, blocks))
    }

    fn lay(&mut self) {
        let width = text_area(out_w(), out_h())[2] as f32;
        self.laid = lay_out(&mut self.adv, &self.blocks, width, GUIDE_SIZES[self.size]);
        self.stale = true;
    }

    fn set_top(&mut self, top: f32) {
        let top = top.clamp(0.0, max_top(&self.laid, out_h())).round();
        if top != self.top {
            self.top = top;
            self.stale = true;
        }
    }

    /// The index of the first line showing.
    fn first(&self) -> usize {
        self.laid
            .lines
            .partition_point(|l| l.y + l.h <= self.top + 0.5)
    }

    pub fn line_down(&mut self) {
        let i = self.first();
        let to = match self.laid.lines.get(i) {
            Some(l) if l.y < self.top - 0.5 => l.y + l.h,
            Some(l) => self.laid.lines.get(i + 1).map_or(l.y + l.h, |n| n.y),
            None => self.top,
        };
        self.set_top(to);
    }

    pub fn line_up(&mut self) {
        let i = self.first();
        let to = match self.laid.lines.get(i) {
            Some(l) if l.y < self.top - 0.5 => l.y,
            _ => i
                .checked_sub(1)
                .and_then(|p| self.laid.lines.get(p))
                .map_or(0.0, |l| l.y),
        };
        self.set_top(to);
    }

    /// The next page: the line cut off at the bottom comes to the top.
    pub fn page_down(&mut self) {
        let area = text_area(out_w(), out_h())[3] as f32;
        let bottom = self.top + area;
        let i = self
            .laid
            .lines
            .partition_point(|l| l.y + l.h <= bottom + 0.5);
        let to =
            self.laid
                .lines
                .get(i)
                .map_or(bottom, |l| if l.y > self.top + 0.5 { l.y } else { bottom });
        self.set_top(to);
    }

    /// The page before: the line at the top goes to the bottom, or just under it.
    pub fn page_up(&mut self) {
        let area = text_area(out_w(), out_h())[3] as f32;
        let want = self.top - area;
        let i = self.laid.lines.partition_point(|l| l.y < want - 0.5);
        let to = self.laid.lines.get(i).map_or(0.0, |l| l.y);
        self.set_top(if to >= self.top { want } else { to });
    }

    /// The text a step smaller or larger, still where it was.
    pub fn resize(&mut self, step: i32) {
        let size = (self.size as i32 + step).clamp(0, GUIDE_SIZES.len() as i32 - 1) as usize;
        if size == self.size {
            return;
        }
        let at = self.laid.src_at(self.top);
        self.size = size;
        self.lay();
        self.top = -1.0;
        self.set_top(self.laid.y_of(at));
    }

    /// Where in the guide's text the page starts.
    pub fn at(&self) -> usize {
        self.laid.src_at(self.top)
    }
}

impl App {
    fn guide_dir(&self) -> Option<PathBuf> {
        Some(self.root()?.join(GUIDES_DIR).join(self.platform.dir_name()))
    }

    fn guide_marks(&mut self) -> &mut GuideMarks {
        if self.guide_marks.is_none() {
            let marks = self.root().map(GuideMarks::load).unwrap_or_default();
            self.guide_marks = Some(marks);
        }
        self.guide_marks.get_or_insert_with(GuideMarks::default)
    }

    fn mark_key(&self, path: &Path) -> String {
        let base = self.root().map(|r| r.join(GUIDES_DIR));
        base.and_then(|b| path.strip_prefix(b).ok().map(Path::to_path_buf))
            .unwrap_or_else(|| path.to_path_buf())
            .to_string_lossy()
            .to_string()
    }

    /// A short MENU press in a game: the game's guide, or a toast when it has none.
    pub(crate) fn open_guide(&mut self) {
        let Some(stem) = self.seated().map(str::to_string) else {
            return;
        };
        let guides = self
            .guide_dir()
            .map(|d| find_guides(&d, &stem))
            .unwrap_or_default();
        if guides.is_empty() {
            self.hud.toast(Toast::NoGuide, self.now());
            return;
        }
        // the guide read last for this game, when it has several
        let index = {
            let keys: Vec<String> = guides.iter().map(|g| self.mark_key(g)).collect();
            let last = self.guide_last.get(&stem).cloned();
            last.and_then(|l| keys.iter().position(|k| *k == l))
                .unwrap_or(0)
        };
        self.show_guide(guides, index, &clean_label(&stem));
    }

    fn show_guide(&mut self, guides: Vec<PathBuf>, index: usize, game: &str) {
        let Some((title, blocks)) = GuideScreen::load(&guides[index]) else {
            self.hud.toast(Toast::NoGuide, self.now());
            return;
        };
        let key = self.mark_key(&guides[index]);
        let marks = self.guide_marks();
        let size = marks.size.unwrap_or(GUIDE_SIZE_DEFAULT);
        let at = marks.at.get(&key).copied().unwrap_or(0);
        let title = if guides.len() > 1 {
            format!("{title}  ({}/{})", index + 1, guides.len())
        } else if title.is_empty() {
            game.to_string()
        } else {
            title
        };
        let mut screen = GuideScreen {
            guides,
            index,
            title,
            blocks,
            adv: Advances::default(),
            glyphs: Glyphs::default(),
            laid: Laid::default(),
            top: -1.0,
            size,
            held: None,
            stale: true,
            moved: None,
        };
        screen.lay();
        screen.set_top(screen.laid.y_of(at));
        if let Some(stem) = self.seated().map(str::to_string) {
            self.guide_last.insert(stem, key);
        }
        self.guide = Some(screen);
    }

    pub fn guide_open(&self) -> bool {
        self.guide.is_some()
    }

    pub fn guide(&self) -> Option<&GuideScreen> {
        self.guide.as_ref()
    }

    /// Keeps where the player is in the open guide, and writes it to the card.
    fn keep_guide_place(&mut self) {
        let Some(screen) = &mut self.guide else {
            return;
        };
        screen.moved = None;
        let (path, at, size) = (
            screen.guides[screen.index].clone(),
            screen.at(),
            screen.size,
        );
        let key = self.mark_key(&path);
        let marks = self.guide_marks();
        marks.at.insert(key, at);
        marks.size = Some(size);
        let marks = marks.clone();
        if let Some(root) = self.root() {
            marks.store(root);
        }
    }

    pub(crate) fn close_guide(&mut self) {
        self.keep_guide_place();
        self.guide = None;
    }

    pub(crate) fn guide_input(&mut self, action: Action) {
        let now = self.now() as f64;
        let Some(screen) = &mut self.guide else {
            return;
        };
        let before = (screen.top, screen.size);
        match action {
            Action::GbaDown(Btn::Down) => {
                screen.line_down();
                screen.held = Some((Btn::Down, now + REPEAT_AFTER_MS));
            }
            Action::GbaDown(Btn::Up) => {
                screen.line_up();
                screen.held = Some((Btn::Up, now + REPEAT_AFTER_MS));
            }
            Action::GbaUp(b @ (Btn::Up | Btn::Down)) => {
                if screen.held.is_some_and(|(h, _)| h == b) {
                    screen.held = None;
                }
            }
            Action::GbaDown(Btn::R1 | Btn::Right) | Action::ShelfRight => screen.page_down(),
            Action::GbaDown(Btn::L1 | Btn::Left) | Action::ShelfLeft => screen.page_up(),
            // L2 and R2 rewind and fast-forward in a handheld's game, and are the game's own
            // in a PlayStation's: either way here they size the text
            Action::GbaDown(Btn::R2) | Action::FfStart => screen.resize(1),
            Action::GbaDown(Btn::L2) | Action::RewindStart => screen.resize(-1),
            Action::GbaDown(Btn::X) if screen.guides.len() > 1 => {
                let (guides, next) = (
                    screen.guides.clone(),
                    (screen.index + 1) % screen.guides.len(),
                );
                self.keep_guide_place();
                let game = self.seated().map(clean_label).unwrap_or_default();
                return self.show_guide(guides, next, &game);
            }
            Action::GbaDown(Btn::B) | Action::QuickMenu | Action::GameMenu => {
                return self.close_guide()
            }
            // a second press of MENU straight after the first: the save states, as ever
            Action::Polaroids => {
                self.close_guide();
                return self.open_polaroids();
            }
            Action::Eject => {
                self.close_guide();
                return self.eject();
            }
            _ => {}
        }
        if (screen.top, screen.size) != before {
            screen.moved = Some(now);
        }
    }

    /// Moves the guide on while Up or Down is held, and writes its place a while after it
    /// last moved.
    pub(crate) fn step_guide(&mut self) {
        if self.guide.is_some() && !matches!(self.phase, crate::app::Phase::Playing { .. }) {
            return self.close_guide();
        }
        let now = self.now() as f64;
        let Some(screen) = &mut self.guide else {
            return;
        };
        if let Some((btn, due)) = screen.held {
            if now >= due {
                match btn {
                    Btn::Down => screen.line_down(),
                    _ => screen.line_up(),
                }
                screen.held = Some((btn, now.max(due) + REPEAT_EVERY_MS));
                screen.moved = Some(now);
            }
        }
        if screen.moved.is_some_and(|m| now - m >= MARK_AFTER_MS) {
            self.keep_guide_place();
        }
    }

    /// The open guide's page, when it has changed since it was last drawn: its pixels, the
    /// screen's size.
    pub fn take_guide_page(&mut self) -> Option<Vec<u8>> {
        let palette = self.palette();
        let screen = self.guide.as_mut()?;
        if !screen.stale {
            return None;
        }
        screen.stale = false;
        let ink = GuideInk::of(palette);
        let keys = if screen.guides.len() > 1 {
            KEYS_MORE
        } else {
            KEYS
        };
        let page = Page {
            title: &screen.title,
            keys,
            ink,
        };
        Some(render_page(
            &mut screen.glyphs,
            &screen.laid,
            screen.top,
            &page,
            out_w(),
            out_h(),
        ))
    }

    pub fn set_guide_tex(&mut self, tex: TexId) {
        self.guide_tex = Some(tex);
    }

    pub(crate) fn draw_guide(&self, out: &mut Vec<Draw>) {
        if let (Some(_), Some(tex)) = (&self.guide, self.guide_tex) {
            out.push(Draw::Tex {
                x: 0.0,
                y: 0.0,
                w: out_w() as f32,
                h: out_h() as f32,
                tex,
                alpha: 1.0,
            });
        }
    }
}
