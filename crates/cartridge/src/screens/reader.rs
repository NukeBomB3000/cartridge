//! The book reader, opened by A on a book: an EPUB's chapters (or a TXT file) set as pages in the
//! guide's type, one text after the other. Left and Right (and L1 and R1) turn a page, Up and Down
//! move a line and keep moving while held, L2 and R2 make the text smaller or larger. SELECT+Y,
//! the same hotkey that picks the picture size in a game, turns the page dark; SELECT+X turns the
//! screen a quarter for reading upright, the d-pad turning with it. Where the player was, the
//! size, the dark and the turn are kept, and a long A on the book starts it again from the top.

use std::path::Path;

use cartridge_gfx::{out_h, out_w, Draw, TexId};
use cartridge_input::{Action, Btn};
use cartridge_store::epub::Epub;
use cartridge_ui::guide::{
    decode, lay_out, max_top, parse_html, parse_txt, render_page, text_area, Advances, Block,
    Glyphs, GuideInk, Laid, Page, GUIDE_SIZES, GUIDE_SIZE_DEFAULT,
};

use crate::app::{App, Phase};
use crate::nav::Dir;

/// How long Up or Down is held before the text keeps moving, and how often it moves then.
const REPEAT_AFTER_MS: f64 = 350.0;
const REPEAT_EVERY_MS: f64 = 55.0;
/// How long the text size's figure shows in the foot after it changed.
const SIZE_SHOWN_MS: f64 = 1500.0;

const SETTINGS_FILE: &str = "Config/reader.ini";
const KEYS: &str = "L1/R1 PAGE   ▲▼ SCROLL   L2/R2 SIZE   SEL+Y DARK   SEL+X TURN   B CLOSE";
/// The same on the narrower page of a turned screen, with less to say.
const KEYS_TURNED: &str = "L1/R1 PAGE   L2/R2 SIZE   SEL+Y DARK   SEL+X TURN";

/// How the screen is turned for reading: none, a quarter clockwise or a quarter back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Turn {
    #[default]
    Upright,
    Clockwise,
    Counter,
}

impl Turn {
    pub fn next(self) -> Turn {
        match self {
            Turn::Upright => Turn::Clockwise,
            Turn::Clockwise => Turn::Counter,
            Turn::Counter => Turn::Upright,
        }
    }

    fn key(self) -> &'static str {
        match self {
            Turn::Upright => "0",
            Turn::Clockwise => "cw",
            Turn::Counter => "ccw",
        }
    }

    fn from_key(key: &str) -> Turn {
        match key {
            "cw" => Turn::Clockwise,
            "ccw" => Turn::Counter,
            _ => Turn::Upright,
        }
    }

    /// The size of the page as it is laid out: a turned screen reads as the other way round.
    pub fn page(self, w: u32, h: u32) -> (u32, u32) {
        match self {
            Turn::Upright => (w, h),
            _ => (h, w),
        }
    }
}

/// What a press of the d-pad does to the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    LineDown,
    LineUp,
    PageNext,
    PagePrev,
}

/// What the d-pad button `btn` does with the screen turned `turn`: as the page is turned, its
/// directions turn with it, so Down still scrolls the text towards its end.
pub fn move_of(btn: Btn, turn: Turn) -> Option<Move> {
    use Btn::{Down, Left, Right, Up};
    Some(match (turn, btn) {
        (Turn::Upright, Down) => Move::LineDown,
        (Turn::Upright, Up) => Move::LineUp,
        (Turn::Upright, Right) => Move::PageNext,
        (Turn::Upright, Left) => Move::PagePrev,
        (Turn::Clockwise, Left) => Move::LineDown,
        (Turn::Clockwise, Right) => Move::LineUp,
        (Turn::Clockwise, Down) => Move::PageNext,
        (Turn::Clockwise, Up) => Move::PagePrev,
        (Turn::Counter, Right) => Move::LineDown,
        (Turn::Counter, Left) => Move::LineUp,
        (Turn::Counter, Up) => Move::PageNext,
        (Turn::Counter, Down) => Move::PagePrev,
        _ => return None,
    })
}

/// `rgba`, `w` by `h`, turned a quarter, and so `h` by `w`.
pub fn turned(rgba: &[u8], w: u32, h: u32, turn: Turn) -> Vec<u8> {
    if turn == Turn::Upright {
        return rgba.to_vec();
    }
    let mut out = vec![0u8; rgba.len()];
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = match turn {
                Turn::Clockwise => (h - 1 - y, x),
                _ => (y, w - 1 - x),
            };
            let from = ((y * w + x) * 4) as usize;
            let to = ((dy * h + dx) * 4) as usize;
            out[to..to + 4].copy_from_slice(&rgba[from..from + 4]);
        }
    }
    out
}

/// The colours of a light page: paper and ink.
fn light() -> GuideInk {
    GuideInk {
        bg: [0xf3, 0xee, 0xe0],
        band: [0xe2, 0xdb, 0xc6],
        body: [0x22, 0x20, 0x1c],
        head: [0x7a, 0x2e, 0x24],
        dim: [0x8a, 0x82, 0x70],
        mono: [0x3a, 0x4a, 0x5a],
    }
}

/// The book while it is open.
pub struct ReaderScreen {
    pub name: String,
    key: String,
    blocks: Vec<Block>,
    adv: Advances,
    glyphs: Glyphs,
    laid: Laid,
    pub top: f32,
    pub size: usize,
    pub dark: bool,
    pub turn: Turn,
    /// Whether the page has changed since it was last drawn.
    pub stale: bool,
    /// A d-pad button held and when the text next moves for it.
    held: Option<(Btn, f64)>,
    /// When the text size last changed, for its figure to show in the foot for a moment.
    size_at: Option<f64>,
}

impl ReaderScreen {
    pub fn new(
        name: &str,
        key: String,
        blocks: Vec<Block>,
        size: usize,
        dark: bool,
        turn: Turn,
    ) -> ReaderScreen {
        ReaderScreen {
            name: name.to_string(),
            key,
            blocks,
            adv: Advances::default(),
            glyphs: Glyphs::default(),
            laid: Laid::default(),
            top: -1.0,
            size,
            dark,
            turn,
            stale: true,
            held: None,
            size_at: None,
        }
    }

    fn lay(&mut self) {
        let (w, h) = self.turn.page(out_w(), out_h());
        self.laid = lay_out(
            &mut self.adv,
            &self.blocks,
            text_area(w, h)[2] as f32,
            GUIDE_SIZES[self.size],
        );
        self.stale = true;
    }

    fn page_height(&self) -> u32 {
        self.turn.page(out_w(), out_h()).1
    }

    /// How far down the text is, as a share (0 to 1) of what there is to scroll.
    pub fn fraction(&self) -> f32 {
        let max = max_top(&self.laid, self.page_height());
        if max <= 0.0 {
            0.0
        } else {
            (self.top / max).clamp(0.0, 1.0)
        }
    }

    /// The index of the line the page starts at: the first that begins at or after its top.
    fn first(&self) -> usize {
        self.laid.lines.partition_point(|l| l.y < self.top - 0.5)
    }

    /// Puts the page's top on the line that starts at `y`, or the nearest line below it, so the
    /// page begins on a whole line. At the end of the text the page is put as low as it goes.
    fn set_top(&mut self, y: f32) {
        let max = max_top(&self.laid, self.page_height());
        let top = if y >= max {
            max
        } else {
            let at = self.laid.lines.partition_point(|l| l.y < y - 0.5);
            self.laid.lines.get(at).map_or(max, |l| l.y).min(max)
        }
        .max(0.0)
        .round();
        if top != self.top {
            self.top = top;
            self.stale = true;
        }
    }

    fn go_to(&mut self, fraction: f32) {
        self.top = -1.0;
        self.set_top(fraction.clamp(0.0, 1.0) * max_top(&self.laid, self.page_height()));
    }

    /// The height of the text's area, and the foot of the page showing.
    fn area(&self) -> f32 {
        text_area(1, self.page_height())[3] as f32
    }

    fn apply(&mut self, m: Move) {
        let lines = &self.laid.lines;
        let first = self.first();
        let to = match m {
            Move::LineDown => lines.get(first + 1).map(|l| l.y),
            Move::LineUp => {
                Some(lines[first.saturating_sub(1).min(lines.len().saturating_sub(1))].y)
            }
            // the first line that does not show whole is where the next page starts
            Move::PageNext => {
                let fit = lines.partition_point(|l| l.y + l.h <= self.top + self.area() + 0.5);
                lines.get(fit.max(first + 1)).map(|l| l.y)
            }
            // the line a page above the top starts at, at the earliest
            Move::PagePrev => {
                let from = lines.partition_point(|l| l.y < self.top - self.area() + 0.5);
                let at = from.min(first.saturating_sub(1));
                lines.get(at).map(|l| l.y)
            }
        };
        match (to, lines.is_empty()) {
            (Some(y), false) => self.set_top(y),
            // past the last line: the page goes as low as it can
            (None, false) if m == Move::LineDown || m == Move::PageNext => self.set_top(f32::MAX),
            _ => {}
        }
    }

    /// The text a step smaller or larger, still where it was.
    fn resize(&mut self, step: i32, now: f64) {
        let size = (self.size as i32 + step).clamp(0, GUIDE_SIZES.len() as i32 - 1) as usize;
        self.size_at = Some(now);
        self.stale = true;
        if size != self.size {
            let at = self.fraction();
            self.size = size;
            self.lay();
            self.go_to(at);
        }
    }

    fn rotate(&mut self) {
        let at = self.fraction();
        self.turn = self.turn.next();
        self.lay();
        self.go_to(at);
    }
}

/// `html` without its XML declaration and its doctype, which the page's parser would print.
pub fn without_prolog(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find("<?").or_else(|| rest.find("<!DOCTYPE")) {
        let open = rest.find("<?").filter(|p| *p == at);
        let close = match open {
            Some(_) => "?>",
            None => ">",
        };
        out.push_str(&rest[..at]);
        match rest[at..].find(close) {
            Some(end) => rest = &rest[at + end + close.len()..],
            None => {
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// An EPUB's chapters, or a TXT file's text, as blocks.
fn load(path: &Path) -> Option<Vec<Block>> {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    if lower.ends_with(".txt") {
        return Some(parse_txt(&decode(&std::fs::read(path).ok()?)));
    }
    let mut epub = Epub::open(path)?;
    let mut blocks = Vec::new();
    for n in 0..epub.chapters() {
        if let Some(html) = epub.chapter(n) {
            blocks.extend(parse_html(&without_prolog(&html)));
        }
    }
    (!blocks.is_empty()).then_some(blocks)
}

impl App {
    /// Opens the book at `path` where it was left, or at its top with a long A.
    pub(crate) fn open_book(&mut self, name: &str, path: &Path, clean: bool) {
        let Some(blocks) = load(path) else {
            eprintln!("cartridge: reader: {} does not read", path.display());
            return;
        };
        let (size, dark, turn) = match &self.root {
            Some(root) => {
                let get = |k: &str| cartridge_store::ini::value(root, SETTINGS_FILE, k);
                (
                    get("size")
                        .and_then(|s| s.parse::<usize>().ok())
                        .filter(|s| *s < GUIDE_SIZES.len())
                        .unwrap_or(GUIDE_SIZE_DEFAULT),
                    get("dark").is_some_and(|d| d == "1"),
                    get("turn").map_or(Turn::Upright, |t| Turn::from_key(&t)),
                )
            }
            None => (GUIDE_SIZE_DEFAULT, false, Turn::Upright),
        };
        let key = self
            .root
            .as_ref()
            .map(|r| crate::media_resume::key(r, path))
            .unwrap_or_default();
        let kept = match &self.root {
            Some(root) if !clean => {
                crate::media_resume::read(root, &key).map(|(_, ppm)| ppm as f32 / 1e6)
            }
            _ => None,
        };
        let mut screen = ReaderScreen::new(name, key, blocks, size, dark, turn);
        screen.lay();
        screen.go_to(kept.unwrap_or(0.0));
        self.reader = Some(screen);
        self.go(Phase::Reader, Dir::Deeper);
    }

    /// Keeps where the player is in the book, and the way they read it.
    pub(crate) fn save_reader_place(&mut self) {
        let (Some(root), Some(screen)) = (&self.root, &self.reader) else {
            return;
        };
        let ppm = (screen.fraction() * 1e6) as u64;
        crate::media_resume::write(root, &screen.key, 0, ppm);
        let set = |k: &str, v: &str| {
            let _ = cartridge_store::ini::write(root, SETTINGS_FILE, k, v);
        };
        set("size", &screen.size.to_string());
        set("dark", if screen.dark { "1" } else { "0" });
        set("turn", screen.turn.key());
    }

    fn close_reader(&mut self) {
        self.save_reader_place();
        self.reader = None;
        self.go(Phase::Shelf, Dir::Back);
    }

    pub(crate) fn reader_input(&mut self, action: Action) {
        let now = self.now() as f64;
        let Some(screen) = &mut self.reader else {
            return self.close_reader();
        };
        match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu | Action::Eject => {
                return self.close_reader()
            }
            Action::PictureToggle => {
                screen.dark = !screen.dark;
                screen.stale = true;
            }
            Action::ShaderNext => screen.rotate(),
            Action::GbaDown(Btn::R1) | Action::ShelfRight => screen.apply(Move::PageNext),
            Action::GbaDown(Btn::L1) | Action::ShelfLeft => screen.apply(Move::PagePrev),
            Action::GbaDown(Btn::R2) | Action::FfStart => screen.resize(1, now),
            Action::GbaDown(Btn::L2) | Action::RewindStart => screen.resize(-1, now),
            Action::GbaDown(btn @ (Btn::Up | Btn::Down | Btn::Left | Btn::Right)) => {
                if let Some(m) = move_of(btn, screen.turn) {
                    screen.apply(m);
                    if matches!(m, Move::LineDown | Move::LineUp) {
                        screen.held = Some((btn, now + REPEAT_AFTER_MS));
                    }
                }
            }
            Action::GbaUp(btn) if screen.held.is_some_and(|(b, _)| b == btn) => {
                screen.held = None;
            }
            _ => {}
        }
    }

    /// Per tick: a button held keeps the text moving.
    pub(crate) fn tick_reader(&mut self) {
        let now = self.now() as f64;
        let Some(screen) = &mut self.reader else {
            return;
        };
        if screen.size_at.is_some_and(|at| now - at > SIZE_SHOWN_MS) {
            screen.size_at = None;
            screen.stale = true;
        }
        if let Some((btn, next)) = screen.held {
            if now >= next {
                if let Some(m) = move_of(btn, screen.turn) {
                    screen.apply(m);
                }
                screen.held = Some((btn, now + REPEAT_EVERY_MS));
            }
        }
    }

    /// The page as it is to be drawn, if it has changed: laid on its own way round, then turned.
    pub fn take_reader_page(&mut self) -> Option<Vec<u8>> {
        let palette = self.palette();
        let battery = self.battery.map(|b| {
            (
                b.percent,
                matches!(b.charge, cartridge_power::Charge::Charging),
            )
        });
        let screen = self.reader.as_mut()?;
        if !screen.stale {
            return None;
        }
        screen.stale = false;
        let ink = if screen.dark {
            GuideInk::of(palette)
        } else {
            light()
        };
        let sized = format!("TEXT SIZE {} / {}", screen.size + 1, GUIDE_SIZES.len());
        let keys = if screen.size_at.is_some() {
            sized.as_str()
        } else if screen.turn == Turn::Upright {
            KEYS
        } else {
            KEYS_TURNED
        };
        let page = Page {
            title: &screen.name,
            keys,
            ink,
            tabs: &[],
            tab: 0,
            battery,
            whole_lines: true,
        };
        let (w, h) = screen.turn.page(out_w(), out_h());
        let rgba = render_page(&mut screen.glyphs, &screen.laid, screen.top, &page, w, h);
        Some(turned(&rgba, w, h, screen.turn))
    }

    pub fn set_reader_tex(&mut self, tex: TexId) {
        self.reader_tex = Some(tex);
    }

    pub(crate) fn draw_reader(&self, out: &mut Vec<Draw>) {
        let (w, h) = (out_w() as f32, out_h() as f32);
        let ground = self.reader.as_ref().map_or([0.0; 3], |r| {
            let bg = if r.dark {
                GuideInk::of(self.palette()).bg
            } else {
                light().bg
            };
            bg.map(|c| f32::from(c) / 255.0)
        });
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
            colour: [ground[0], ground[1], ground[2], 1.0],
        });
        if let (Some(_), Some(tex)) = (&self.reader, self.reader_tex) {
            out.push(Draw::Tex {
                x: 0.0,
                y: 0.0,
                w,
                h,
                tex,
                alpha: 1.0,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_d_pad_turns_with_the_page_so_down_still_scrolls_towards_the_end() {
        assert_eq!(move_of(Btn::Down, Turn::Upright), Some(Move::LineDown));
        assert_eq!(move_of(Btn::Left, Turn::Clockwise), Some(Move::LineDown));
        assert_eq!(move_of(Btn::Right, Turn::Counter), Some(Move::LineDown));
        // every turn has each of the four directions for one of the four moves
        for turn in [Turn::Upright, Turn::Clockwise, Turn::Counter] {
            let mut moves: Vec<Move> = [Btn::Up, Btn::Down, Btn::Left, Btn::Right]
                .into_iter()
                .filter_map(|b| move_of(b, turn))
                .collect();
            moves.sort_by_key(|m| *m as u8);
            moves.dedup();
            assert_eq!(moves.len(), 4, "{turn:?}");
        }
        assert_eq!(move_of(Btn::A, Turn::Upright), None);
    }

    #[test]
    fn a_turned_page_is_the_other_way_round_and_each_pixel_is_where_the_turn_puts_it() {
        // 3 by 2, the first pixel red
        let mut rgba = vec![0u8; 3 * 2 * 4];
        rgba[0..4].copy_from_slice(&[255, 0, 0, 255]);
        let cw = turned(&rgba, 3, 2, Turn::Clockwise);
        // a quarter clockwise puts the top left corner at the top right, 2 by 3 now
        assert_eq!(&cw[(1 * 4)..(1 * 4 + 4)], &[255, 0, 0, 255]);
        let ccw = turned(&rgba, 3, 2, Turn::Counter);
        // a quarter back puts it at the bottom left
        let at = (2 * 2) * 4;
        assert_eq!(&ccw[at..at + 4], &[255, 0, 0, 255]);
        assert_eq!(turned(&rgba, 3, 2, Turn::Upright), rgba);
        assert_eq!(Turn::Clockwise.page(640, 480), (480, 640));
        assert_eq!(Turn::Upright.page(640, 480), (640, 480));
    }

    fn long_text() -> String {
        (1..=120)
            .map(|i| {
                format!("Line number {i} of a long enough text to fill the page and then some.")
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    #[test]
    fn every_move_puts_the_page_on_the_start_of_a_line_and_the_end_on_the_last() {
        for turn in [Turn::Upright, Turn::Clockwise] {
            let mut r =
                ReaderScreen::new("T", String::new(), parse_txt(&long_text()), 2, false, turn);
            r.lay();
            r.go_to(0.0);
            let starts: Vec<f32> = r.laid.lines.iter().map(|l| l.y).collect();
            let max = max_top(&r.laid, r.page_height());
            assert!(max > 0.0, "the text is longer than a page");
            let on_a_line = |top: f32| starts.iter().any(|y| (*y - top).abs() < 1.0) || top == max;
            let mut seen = Vec::new();
            for m in [
                Move::PageNext,
                Move::LineDown,
                Move::PageNext,
                Move::LineDown,
                Move::PagePrev,
                Move::LineUp,
            ] {
                r.apply(m);
                assert!(
                    on_a_line(r.top),
                    "{turn:?} {m:?}: the top {} is between lines",
                    r.top
                );
                seen.push(r.top);
            }
            assert!(seen.windows(2).any(|w| w[0] != w[1]), "the moves move");
            // to the end and no further: the last page is as low as it goes
            for _ in 0..400 {
                r.apply(Move::PageNext);
            }
            assert_eq!(r.top, max.round());
            r.apply(Move::LineDown);
            assert_eq!(r.top, max.round(), "no further");
            // and back to the top
            for _ in 0..400 {
                r.apply(Move::PagePrev);
            }
            assert_eq!(r.top, 0.0);
        }
    }

    #[test]
    fn a_page_never_cuts_a_line_at_its_foot() {
        let mut r = ReaderScreen::new(
            "T",
            String::new(),
            parse_txt(&long_text()),
            2,
            false,
            Turn::Upright,
        );
        r.lay();
        r.go_to(0.3);
        let area = r.area();
        // the lines the page shows whole end inside its area, and the next one does not fit
        let shown: Vec<_> = r
            .laid
            .lines
            .iter()
            .filter(|l| l.y >= r.top - 0.5 && l.y + l.h <= r.top + area + 0.5)
            .collect();
        assert!(!shown.is_empty());
        assert!(shown.iter().all(|l| l.y + l.h <= r.top + area + 0.5));
    }

    #[test]
    fn a_text_file_is_read_as_a_book_and_a_file_that_is_neither_is_not() {
        let d = tempfile::tempdir().unwrap();
        let txt = d.path().join("Story.txt");
        std::fs::write(&txt, "Chapter One\n\nIt was a dark and stormy night.\n").unwrap();
        assert!(load(&txt).is_some_and(|b| !b.is_empty()));
        let bad = d.path().join("Broken.epub");
        std::fs::write(&bad, b"nope").unwrap();
        assert!(load(&bad).is_none());
    }

    #[test]
    fn the_xml_declaration_and_the_doctype_are_taken_off_a_chapter() {
        let html = "<?xml version='1.0' encoding='utf-8'?>\n<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.1//EN\" \"x\">\n<html><body><p>Hi</p></body></html>";
        assert_eq!(
            without_prolog(html).trim(),
            "<html><body><p>Hi</p></body></html>"
        );
        assert_eq!(without_prolog("<p>plain</p>"), "<p>plain</p>");
    }

    #[test]
    fn the_turns_go_round_and_are_kept_by_name() {
        let mut t = Turn::Upright;
        for _ in 0..3 {
            t = t.next();
        }
        assert_eq!(t, Turn::Upright);
        for turn in [Turn::Upright, Turn::Clockwise, Turn::Counter] {
            assert_eq!(Turn::from_key(turn.key()), turn);
        }
    }
}
