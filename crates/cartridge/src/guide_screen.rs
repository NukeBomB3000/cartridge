//! The game menu a short MENU press opens over a game, in three tabs that L1 and R1 turn
//! between: the game's guide, the game's achievements and the settings. The guide is a
//! walkthrough the player keeps in GUIDES/<system>/ named as the game is, TXT or HTML, read over
//! the paused game. Left and Right turn a page, Up and Down move a line and keep moving while held,
//! L2 and R2 make the text smaller or larger, X goes on to the game's next guide when it has
//! several (kept in a folder named as the game), and B or MENU close it. The settings tab is a
//! list: Up and Down choose a row, Right steps it on and Left back; it holds what the hotkeys change
//! (brightness, volume, screen look and more) and the options of the core running the game. A
//! second MENU press straight after the first is still a double press and opens the save
//! states.
//!
//! Where the player was in each guide, and the size they read at, are kept in
//! Config/guides.txt, so the guide opens where it was left, at any size.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use cartridge_gfx::{out_h, out_w, Draw, TexId};
use cartridge_input::{Action, Btn};
use cartridge_retro::CoreOption;
use cartridge_store::{Platform, BLUE_LIGHT_MAX, BRIGHTNESS_MAX};
use cartridge_ui::guide::{
    decode, is_guide_file, lay_out, max_top, parse, render_page, text_area, Advances, Block,
    Glyphs, GuideInk, Laid, Page, Word, GUIDE_SIZES, GUIDE_SIZE_DEFAULT,
};
use cartridge_ui::{clean_label, SettingsRow};

use crate::app::App;

/// The folder on the card the guides are kept in, a folder for each system under it.
pub const GUIDES_DIR: &str = "GUIDES";
const MARKS_FILE: &str = "guides.txt";

/// How long Up or Down is held before the guide keeps moving, and how often it moves then.
const REPEAT_AFTER_MS: f64 = 350.0;
const REPEAT_EVERY_MS: f64 = 55.0;

/// How long after the place in a guide last moved it is written to the card.
const MARK_AFTER_MS: f64 = 2000.0;

pub const KEYS: &str = "L1/R1 TAB   ◀▶ PAGE   ▲▼ SCROLL   L2/R2 SIZE   B BACK";
pub const KEYS_MORE: &str = "L1/R1 TAB   ◀▶ PAGE   ▲▼ SCROLL   L2/R2 SIZE   X NEXT GUIDE   B BACK";
pub const KEYS_ACHIEVEMENTS: &str = "L1/R1 TAB   ◀▶ PAGE   ▲▼ SCROLL   L2/R2 SIZE   B BACK";
pub const KEYS_SETTINGS: &str = "L1/R1 TAB   ▲▼ CHOOSE   ◀▶ CHANGE   B BACK";

/// The tabs the menu turns between.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Guide,
    Achievements,
    Settings,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Guide, Tab::Achievements, Tab::Settings];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Guide => "GUIDE",
            Tab::Achievements => "ACHIEVEMENTS",
            Tab::Settings => "SETTINGS",
        }
    }

    fn index(self) -> usize {
        Tab::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    /// The tab after or before this one, round from either end.
    fn step(self, forward: bool) -> Tab {
        let n = Tab::ALL.len();
        Tab::ALL[(self.index() + if forward { 1 } else { n - 1 }) % n]
    }
}

/// The settings the hotkeys change, as rows of the settings tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quick {
    Brightness,
    BlueLight,
    Volume,
    Look,
    Picture,
    Palette,
    Ghosting,
    Colour,
    Rumble,
}

/// A row of the settings tab: a setting of the handheld, or an option of the core.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    Quick(Quick),
    Core(usize),
}

fn words(text: &str, bold: bool, src: usize) -> Vec<Word> {
    text.split_whitespace()
        .map(|w| Word {
            text: w.to_string(),
            bold,
            src,
        })
        .collect()
}

fn para(text: &str, src: usize) -> Block {
    Block::Para {
        words: words(text, false, src),
        indent: 0,
    }
}

fn heading(text: &str, src: usize) -> Block {
    Block::Heading {
        words: words(text, true, src),
        level: 2,
    }
}

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
    /// The tab open.
    pub tab: Tab,
    /// What the tab open shows, laid out in `laid`.
    blocks: Vec<Block>,
    /// The guide's own text, kept while another tab shows, and where it was left.
    guide_blocks: Vec<Block>,
    guide_top: f32,
    /// How many badges had come in when the achievements were last laid out.
    badges_seen: u64,
    /// The settings row the cursor is on, and what the rows are.
    sel: usize,
    rows: Vec<Row>,
    /// The core's options, once it has told them; none before.
    options: Option<Vec<CoreOption>>,
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

    /// All the words the open tab shows, in one string.
    pub fn text_of_tab(&self) -> String {
        self.blocks
            .iter()
            .flat_map(|b| match b {
                Block::Para { words, .. }
                | Block::Heading { words, .. }
                | Block::Item { words, .. } => words.clone(),
                Block::Entry { title, detail, .. } => [title.clone(), detail.clone()].concat(),
                _ => Vec::new(),
            })
            .map(|w| w.text)
            .collect::<Vec<_>>()
            .join(" ")
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
        let (title, blocks) = match guides.get(index).and_then(|g| GuideScreen::load(g)) {
            Some(loaded) => loaded,
            None => (String::new(), self.no_guide_blocks(game)),
        };
        let key = guides
            .get(index)
            .map(|g| self.mark_key(g))
            .unwrap_or_default();
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
            tab: Tab::Guide,
            guide_blocks: blocks.clone(),
            guide_top: 0.0,
            badges_seen: 0,
            sel: 0,
            rows: Vec::new(),
            options: None,
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
            if !key.is_empty() {
                self.guide_last.insert(stem, key);
            }
        }
        self.guide = Some(screen);
    }

    /// What the guide tab says of a game that has none: where to put one, and exactly what the
    /// file is to be called.
    fn no_guide_blocks(&self, game: &str) -> Vec<Block> {
        let stem = self.seated().unwrap_or(game).to_string();
        let dir = self.platform.dir_name();
        vec![
            heading(&format!("No guide for {game}"), 0),
            para("Put a walkthrough on the card as a TXT or HTML file named just like the game:", 1),
            Block::Item {
                words: words(&format!("GUIDES/{dir}/{stem}.txt"), true, 2),
                indent: 0,
                mark: "-".to_string(),
            },
            Block::Item {
                words: words(&format!("GUIDES/{dir}/{stem}.html"), true, 3),
                indent: 0,
                mark: "-".to_string(),
            },
            para(
                &format!(
                    "Several guides for one game go in a folder with that name, GUIDES/{dir}/{stem}/. Capitals, brackets and punctuation in the name do not matter."
                ),
                4,
            ),
        ]
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
        if screen.tab != Tab::Guide || screen.guides.is_empty() {
            return;
        }
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
        let Some(tab) = self.guide.as_ref().map(|g| g.tab) else {
            return;
        };
        // L1 and R1 turn the tabs; the settings tab is a list that Up and Down choose in and
        // Left and Right change, as in the other menus
        match action {
            Action::GbaDown(Btn::R1) | Action::ShelfRight => return self.guide_switch(true),
            Action::GbaDown(Btn::L1) | Action::ShelfLeft => return self.guide_switch(false),
            Action::GbaDown(Btn::Right) if tab == Tab::Settings => return self.guide_change(true),
            Action::GbaDown(Btn::Left) if tab == Tab::Settings => return self.guide_change(false),
            Action::GbaDown(Btn::Down) if tab == Tab::Settings => return self.guide_choose(true),
            Action::GbaDown(Btn::Up) if tab == Tab::Settings => return self.guide_choose(false),
            _ => {}
        }
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
            Action::GbaDown(Btn::X) if tab == Tab::Guide && screen.guides.len() > 1 => {
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
        // a badge that came in since the list was laid out: lay it out again with it
        if self.guide.as_ref().is_some_and(|g| {
            g.tab == Tab::Achievements && g.badges_seen != crate::cheevos::badge_version()
        }) {
            self.fill_tab();
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

    /// Turns to the next tab or the last, keeping the place in the guide.
    fn guide_switch(&mut self, forward: bool) {
        self.keep_guide_place();
        let Some(screen) = &mut self.guide else {
            return;
        };
        if screen.tab == Tab::Guide {
            screen.guide_top = screen.top;
        }
        screen.tab = screen.tab.step(forward);
        screen.held = None;
        screen.sel = 0;
        self.fill_tab();
    }

    /// Lays out what the open tab shows: the guide's own text, the game's achievements or the
    /// settings, the guide at the place it was left and the others from the top.
    fn fill_tab(&mut self) {
        let Some(tab) = self.guide.as_ref().map(|g| g.tab) else {
            return;
        };
        let seen = crate::cheevos::badge_version();
        let blocks = match tab {
            Tab::Guide => None,
            Tab::Achievements => Some(self.achievement_blocks()),
            Tab::Settings => Some(self.settings_blocks()),
        };
        let Some(screen) = &mut self.guide else {
            return;
        };
        screen.blocks = blocks.unwrap_or_else(|| screen.guide_blocks.clone());
        screen.badges_seen = seen;
        let keep = if tab == Tab::Guide {
            screen.guide_top
        } else {
            screen.top.max(0.0)
        };
        screen.lay();
        screen.top = -1.0;
        screen.set_top(keep);
        if tab == Tab::Settings {
            self.show_settings_row();
        }
    }

    /// The achievements of the game running, unlocked first.
    fn achievement_blocks(&self) -> Vec<Block> {
        let status = crate::cheevos::status();
        if !self.state.cheevos {
            return vec![para(
                "Achievements are off. Switch them on in Settings > Achievements.",
                0,
            )];
        }
        if matches!(status, crate::cheevos::Status::SignedOut) {
            return vec![para(
                "Sign in to RetroAchievements in Settings > Achievements and your games are followed.",
                0,
            )];
        }
        let list = crate::cheevos::achievements();
        if list.is_empty() {
            return vec![para(
                "No achievements for this game: it may not be known to RetroAchievements, or they are still coming in.",
                0,
            )];
        }
        let done = list.iter().filter(|a| a.unlocked).count();
        let (points, total) = (
            list.iter()
                .filter(|a| a.unlocked)
                .map(|a| a.points)
                .sum::<u32>(),
            list.iter().map(|a| a.points).sum::<u32>(),
        );
        let mut blocks = vec![heading(
            &format!(
                "{done} of {} unlocked, {points} of {total} points",
                list.len()
            ),
            0,
        )];
        for (i, a) in list.iter().enumerate() {
            let src = (i + 1) * 10;
            blocks.push(Block::Entry {
                icon: crate::cheevos::badge(&a.badge_url),
                title: words(&format!("{} ({})", a.title, a.points), true, src),
                detail: words(&a.description, false, src + 1),
                locked: !a.unlocked,
            });
        }
        blocks
    }

    /// The handheld's settings that suit a game on its way, in the order the tab lists them.
    fn quick_rows(&self) -> Vec<Quick> {
        let mut rows = vec![
            Quick::Brightness,
            Quick::BlueLight,
            Quick::Volume,
            Quick::Look,
            Quick::Picture,
        ];
        if self.platform == Platform::Gb {
            rows.push(Quick::Palette);
        }
        rows.push(Quick::Ghosting);
        if self.platform.colour_correction() {
            rows.push(Quick::Colour);
        }
        rows.push(Quick::Rumble);
        rows
    }

    fn quick_text(&self, quick: Quick) -> (&'static str, String) {
        let on = |b: bool| if b { "On" } else { "Off" }.to_string();
        match quick {
            Quick::Brightness => (
                "Brightness",
                format!("{} / {}", self.state.brightness, BRIGHTNESS_MAX),
            ),
            Quick::BlueLight => (
                "Blue light",
                format!("{} / {}", self.state.blue_light, BLUE_LIGHT_MAX),
            ),
            Quick::Volume => (
                "Volume",
                if self.muted() {
                    "Muted".to_string()
                } else {
                    format!("{}", self.level())
                },
            ),
            Quick::Look => ("Screen look", format!("{:?}", self.look())),
            Quick::Picture => ("Picture size", self.video_mode.as_str().to_string()),
            Quick::Palette => ("Palette", self.state.gb_palette.label().to_string()),
            Quick::Ghosting => ("Ghosting", on(self.ghosting_on())),
            Quick::Colour => ("Colour correction", on(self.state.colour_correction)),
            Quick::Rumble => (
                "Rumble",
                self.settings_value(SettingsRow::Rumble)
                    .map(|v| v.text().to_string())
                    .unwrap_or_default(),
            ),
        }
    }

    /// The options of the core that the tab shows.
    fn core_rows(&self) -> Vec<CoreOption> {
        let core = self.core();
        self.guide
            .as_ref()
            .and_then(|g| g.options.clone())
            .unwrap_or_default()
            .into_iter()
            .filter(|o| crate::core_options::shown(core, o))
            .collect()
    }

    /// The settings tab's text, and what its rows are, with the cursor row in bold.
    fn settings_blocks(&mut self) -> Vec<Block> {
        let quick = self.quick_rows();
        let core = self.core_rows();
        let sel = self.guide.as_ref().map_or(0, |g| g.sel);
        let mut rows: Vec<Row> = quick.iter().map(|q| Row::Quick(*q)).collect();
        rows.extend((0..core.len()).map(Row::Core));
        let mut blocks = vec![heading("Handheld", 0)];
        let mut core_heading = false;
        for (i, row) in rows.iter().enumerate() {
            let (label, value) = match row {
                Row::Quick(q) => {
                    let (l, v) = self.quick_text(*q);
                    (l.to_string(), v)
                }
                Row::Core(c) => (core[*c].label.clone(), core[*c].value.clone()),
            };
            if matches!(row, Row::Core(_)) && !core_heading {
                core_heading = true;
                blocks.push(heading(
                    &format!("{} options", self.core().name()),
                    (i + 1) * 10 - 5,
                ));
            }
            blocks.push(Block::Item {
                words: words(&format!("{label}: {value}"), i == sel, (i + 1) * 10),
                indent: 0,
                mark: if i == sel { ">" } else { " " }.to_string(),
            });
        }
        if self.guide.as_ref().is_some_and(|g| g.options.is_none()) {
            blocks.push(para("Loading the core's options...", 100_000));
        }
        if let Some(screen) = &mut self.guide {
            screen.rows = rows;
        }
        blocks
    }

    /// Scrolls the settings tab so the row the cursor is on shows.
    fn show_settings_row(&mut self) {
        let Some(screen) = &mut self.guide else {
            return;
        };
        let src = (screen.sel + 1) * 10;
        let y = screen.laid.y_of(src);
        let area = text_area(out_w(), out_h())[3] as f32;
        let line = screen.laid.px * 1.4;
        if y < screen.top {
            screen.set_top(y - line);
        } else if y + line * 2.0 > screen.top + area {
            screen.set_top(y + line * 2.0 - area);
        }
    }

    /// Moves the settings cursor a row, round from either end.
    fn guide_choose(&mut self, down: bool) {
        let Some(screen) = &mut self.guide else {
            return;
        };
        let n = screen.rows.len();
        if n == 0 {
            return;
        }
        screen.sel = if down {
            (screen.sel + 1) % n
        } else {
            (screen.sel + n - 1) % n
        };
        self.fill_tab();
    }

    /// Steps the setting the cursor is on to its next value, or its last.
    fn guide_change(&mut self, forward: bool) {
        let Some((row, sel)) = self
            .guide
            .as_ref()
            .and_then(|g| g.rows.get(g.sel).map(|r| (*r, g.sel)))
        else {
            return;
        };
        match row {
            Row::Quick(q) => self.quick_step(q, forward),
            Row::Core(i) => {
                let Some(option) = self.core_rows().get(i).cloned() else {
                    return;
                };
                let count = option.values.len();
                if count < 2 {
                    return;
                }
                let at = option
                    .values
                    .iter()
                    .position(|v| *v == option.value)
                    .unwrap_or(0);
                let to = (at + if forward { 1 } else { count - 1 }) % count;
                let value = option.values[to].clone();
                if let Some(opts) = self.guide.as_mut().and_then(|g| g.options.as_mut()) {
                    if let Some(o) = opts.iter_mut().find(|o| o.key == option.key) {
                        o.value = value.clone();
                    }
                }
                if let Some(root) = self.root.as_deref() {
                    if let Err(e) = cartridge_store::ini::write(
                        root,
                        crate::core_options::CORE_OPTIONS_FILE,
                        &option.key,
                        &value,
                    ) {
                        eprintln!("cartridge: {}: {e}", crate::core_options::CORE_OPTIONS_FILE);
                    }
                }
                self.core_option_pending = Some((option.key, value));
            }
        }
        if let Some(screen) = &mut self.guide {
            screen.sel = sel;
        }
        self.fill_tab();
    }

    fn quick_step(&mut self, quick: Quick, forward: bool) {
        let up_or_down = |up, down| if forward { up } else { down };
        match quick {
            Quick::Brightness => {
                self.adjust(up_or_down(Action::BrightnessUp, Action::BrightnessDown));
            }
            Quick::BlueLight => {
                self.adjust(up_or_down(Action::BlueLightUp, Action::BlueLightDown));
            }
            Quick::Volume => {
                self.adjust(up_or_down(Action::VolumeUp, Action::VolumeDown));
            }
            Quick::Look => {
                self.adjust(Action::ShaderNext);
            }
            Quick::Picture => {
                self.adjust(Action::PictureToggle);
            }
            Quick::Palette => {
                self.adjust(up_or_down(Action::PaletteNext, Action::PalettePrev));
            }
            Quick::Ghosting => {
                self.adjust(Action::LcdGhostingToggle);
            }
            Quick::Colour => {
                self.adjust(Action::ColourCorrectionToggle);
            }
            Quick::Rumble => self.change_setting(SettingsRow::Rumble, forward),
        }
    }

    /// Whether the settings tab waits for the core's options.
    pub fn wants_guide_options(&self) -> bool {
        self.guide
            .as_ref()
            .is_some_and(|g| g.tab == Tab::Settings && g.options.is_none())
    }

    /// The core's options as it reports them, for the settings tab.
    pub fn set_guide_options(&mut self, options: Vec<CoreOption>) {
        if let Some(screen) = &mut self.guide {
            screen.options = Some(options);
        }
        self.fill_tab();
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
        let keys = match screen.tab {
            Tab::Guide if screen.guides.len() > 1 => KEYS_MORE,
            Tab::Guide => KEYS,
            Tab::Achievements => KEYS_ACHIEVEMENTS,
            Tab::Settings => KEYS_SETTINGS,
        };
        let tabs = Tab::ALL.map(Tab::label);
        let title = if screen.tab == Tab::Guide {
            screen.title.as_str()
        } else {
            ""
        };
        let page = Page {
            title,
            keys,
            ink,
            tabs: &tabs,
            tab: screen.tab.index(),
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
