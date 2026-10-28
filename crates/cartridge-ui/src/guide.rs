//! The game guide: a walkthrough kept as TXT or HTML next to the games, read over a paused game.
//!
//! A TXT guide is mostly prose wrapped by hand at some 80 columns, with tables, maps and
//! ASCII art between. The prose is joined back into paragraphs that wrap to the screen, a word
//! split over two lines by a hyphen whole again; tables and art keep their columns, set in a
//! fixed-width face and shrunk until they fit across. An HTML guide gives its headings,
//! paragraphs, lists, tables and preformatted blocks for what they are; scripts, styles and
//! the page around the guide are left out.
//!
//! Every word remembers where it is in the file, so a place in the guide outlives the size of
//! the text it was read at.

use crate::kit::palette::Palette;
use std::collections::HashMap;
use std::sync::OnceLock;

use fontdue::{Font, FontSettings, Metrics};

const SANS_TTF: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");
const MONO_TTF: &[u8] = include_bytes!("../assets/DejaVuSansMono.ttf");

/// The sizes the text can be read at, in screen pixels, L2 and R2 stepping through them.
pub const GUIDE_SIZES: [f32; 8] = [12.0, 14.0, 16.0, 18.0, 20.0, 23.0, 26.0, 30.0];
pub const GUIDE_SIZE_DEFAULT: usize = 2;

/// The smallest a table or a piece of art is shrunk to so it fits across.
const MONO_LEAST: f32 = 8.0;

/// The band across the top with the guide's name and how far in it is, and the one across the
/// bottom with the keys, in screen pixels.
pub const GUIDE_HEAD_H: u32 = 26;
pub const GUIDE_FOOT_H: u32 = 22;

/// The guide's colours, taken from the theme's: the page as dark as the theme goes so the text
/// stands out, the bands a panel's well, the text its cream, and headings, tables and art in the
/// theme's lighter tones.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct GuideInk {
    pub bg: [u8; 3],
    pub band: [u8; 3],
    pub body: [u8; 3],
    pub head: [u8; 3],
    pub dim: [u8; 3],
    pub mono: [u8; 3],
}

impl GuideInk {
    pub fn of(pal: &Palette) -> GuideInk {
        GuideInk {
            bg: mix(pal.dark, [0, 0, 0], 0.35),
            band: pal.well,
            body: pal.cream,
            head: pal.label,
            dim: pal.dim,
            mono: mix(pal.cream, pal.frame_hi, 0.4),
        }
    }
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2])]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    pub bold: bool,
    /// Where the word starts in the guide's text.
    pub src: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Para {
        words: Vec<Word>,
        indent: u8,
    },
    Heading {
        words: Vec<Word>,
        level: u8,
    },
    Item {
        words: Vec<Word>,
        indent: u8,
        mark: String,
    },
    /// Lines kept as they are, each with where it starts.
    Pre {
        lines: Vec<(String, usize)>,
    },
    Rule {
        src: usize,
    },
    /// An entry in a list that has a picture: its title and what it says under it, set beside
    /// the picture, a square of 64 by 64 RGBA pixels shown at a few lines' height. `locked` sets
    /// it in the dim ink.
    Entry {
        icon: Option<std::sync::Arc<Vec<u8>>>,
        title: Vec<Word>,
        detail: Vec<Word>,
        locked: bool,
    },
}

/// How wide and tall an entry's picture is shown at text size `px`.
pub fn entry_icon_px(px: f32) -> f32 {
    (px * 2.6).round()
}

impl Block {
    /// Where the block starts in the guide's text.
    pub fn src(&self) -> usize {
        match self {
            Block::Para { words, .. }
            | Block::Heading { words, .. }
            | Block::Item { words, .. } => words.first().map_or(0, |w| w.src),
            Block::Pre { lines } => lines.first().map_or(0, |l| l.1),
            Block::Rule { src } => *src,
            Block::Entry { title, .. } => title.first().map_or(0, |w| w.src),
        }
    }
}

/// A guide's bytes as text: UTF-8 when they are, else the Windows code page most old guides
/// were typed in. Carriage returns go, and tabs become spaces to the next eighth column.
pub fn decode(bytes: &[u8]) -> String {
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s.trim_start_matches('\u{feff}').to_string(),
        Err(_) => bytes.iter().map(|&b| cp1252(b)).collect(),
    };
    let mut out = String::with_capacity(text.len());
    let mut col = 0;
    for ch in text.chars() {
        match ch {
            '\r' => {}
            '\n' => {
                out.push('\n');
                col = 0;
            }
            '\t' => {
                let to = (col / 8 + 1) * 8;
                out.extend(std::iter::repeat_n(' ', to - col));
                col = to;
            }
            c => {
                out.push(c);
                col += 1;
            }
        }
    }
    out
}

fn cp1252(b: u8) -> char {
    const HIGH: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž',
        '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}',
        'ž', 'Ÿ',
    ];
    match b {
        0x80..=0x9f => HIGH[(b - 0x80) as usize],
        _ => b as char,
    }
}

/// Whether `name` is a guide's file name: TXT or HTML.
pub fn is_guide_file(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [".txt", ".html", ".htm"].iter().any(|e| lower.ends_with(e))
}

/// The guide in `text`, read as HTML when `html`, else as plain text.
pub fn parse(text: &str, html: bool) -> Vec<Block> {
    if html {
        parse_html(text)
    } else {
        parse_txt(text)
    }
}

// ---------------------------------------------------------------- plain text

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Blank,
    Prose,
    Art,
    Rule,
}

struct Raw<'a> {
    text: &'a str,
    at: usize,
    indent: usize,
    kind: Kind,
}

const RULE_CHARS: &str = "-=_*~#+.";
const DECOR: &str = "~=-*#+";

fn is_rule(t: &str) -> bool {
    let mut chars = t.chars().filter(|c| *c != ' ');
    let Some(first) = chars.clone().next() else {
        return false;
    };
    t.chars().filter(|c| *c != ' ').count() >= 4
        && RULE_CHARS.contains(first)
        && chars.all(|c| RULE_CHARS.contains(c))
}

/// Whether a line holds columns or a drawing rather than words: a run of spaces between
/// things, a box's edges, a long run of drawing characters, or more of those than letters.
fn is_art(t: &str) -> bool {
    if t.contains("   ") {
        return true;
    }
    if t.starts_with('|') || t.ends_with('|') || t.starts_with("+-") || t.contains("-+-") {
        return true;
    }
    for run in ["----", "====", "____", "~~~~", "****", "####", "...."] {
        if t.contains(run) {
            return true;
        }
    }
    let n = t.chars().count();
    let odd = t
        .chars()
        .filter(|c| {
            !c.is_alphanumeric() && !c.is_whitespace() && !".,;:!?'\"()-–’„“”…/&%".contains(*c)
        })
        .count();
    n >= 3 && odd * 10 > n * 3
}

/// The words of a line decorated as a heading, "~~ Champa ~~" or "=== WALKTHROUGH ===".
fn decorated(t: &str) -> Option<&str> {
    let first = t.chars().next()?;
    if !DECOR.contains(first) || !t.ends_with(first) {
        return None;
    }
    let inner = t.trim_matches(|c: char| DECOR.contains(c) || c == ' ');
    let lead = t.len() - t.trim_start_matches(first).len();
    (lead >= 2 && inner.chars().any(char::is_alphabetic) && !inner.contains(first)).then_some(inner)
}

/// A list item's mark and where its words start, for "- x", "* x", "• x", "1. x", "a) x".
fn item_mark(t: &str) -> Option<(String, usize)> {
    for m in ["- ", "* ", "• ", "o ", "+ ", "> "] {
        if t.starts_with(m) && t.len() > m.len() && !t[m.len()..].starts_with(' ') {
            return Some(("•".into(), m.len()));
        }
    }
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    let alpha = t.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && t[1..].starts_with([')', '.'])
        && t.len() > 2;
    let lead = if digits > 0 && digits <= 3 {
        digits
    } else if alpha {
        1
    } else {
        return None;
    };
    let rest = &t[lead..];
    if (rest.starts_with(". ") || rest.starts_with(") "))
        && rest.len() > 2
        && !rest[2..].starts_with(' ')
    {
        return Some((t[..lead + 1].to_string(), lead + 2));
    }
    None
}

fn words_of(text: &str, at: usize, bold: bool, out: &mut Vec<Word>) {
    let mut start = None;
    for (i, c) in text
        .char_indices()
        .chain(std::iter::once((text.len(), ' ')))
    {
        match (c.is_whitespace(), start) {
            (true, Some(s)) => {
                out.push(Word {
                    text: text[s..i].to_string(),
                    bold,
                    src: at + s,
                });
                start = None;
            }
            (false, None) => start = Some(i),
            _ => {}
        }
    }
}

/// Adds a hard-wrapped line's words to a paragraph's, a word split by a hyphen at the end of
/// the last line made whole again ("vor-" "geben").
fn join_line(words: &mut Vec<Word>, text: &str, at: usize) {
    let mut next = Vec::new();
    words_of(text, at, false, &mut next);
    if let (Some(last), Some(first)) = (words.last_mut(), next.first()) {
        let mut tail = last.text.chars().rev();
        let split = tail.next() == Some('-')
            && tail.next().is_some_and(|c| c.is_lowercase())
            && first.text.chars().next().is_some_and(|c| c.is_lowercase());
        if split {
            last.text.pop();
            last.text.push_str(&first.text);
            next.remove(0);
        }
    }
    words.extend(next);
}

pub fn parse_txt(text: &str) -> Vec<Block> {
    let mut raws: Vec<Raw> = Vec::new();
    let mut at = 0;
    for line in text.split('\n') {
        let body = line.trim_end();
        let t = body.trim_start();
        let kind = if t.is_empty() {
            Kind::Blank
        } else if is_rule(t) {
            Kind::Rule
        } else if decorated(t).is_none() && is_art(t) {
            Kind::Art
        } else {
            Kind::Prose
        };
        raws.push(Raw {
            text: body,
            at,
            indent: body.len() - t.len(),
            kind,
        });
        at += line.len() + 1;
    }
    let least = raws
        .iter()
        .filter(|r| r.kind != Kind::Blank)
        .map(|r| r.indent)
        .min()
        .unwrap_or(0);
    // the column the prose was wrapped at: most of its lines come close to it
    let mut lens: Vec<usize> = raws
        .iter()
        .filter(|r| r.kind == Kind::Prose)
        .map(|r| r.text.chars().count())
        .collect();
    lens.sort_unstable();
    let wrap = lens.get(lens.len() * 9 / 10).copied().unwrap_or(78).max(40);

    // a rule against a table or drawing is part of it
    for i in 0..raws.len() {
        if raws[i].kind == Kind::Rule {
            let near_art = |j: Option<usize>| {
                j.and_then(|j| raws.get(j))
                    .is_some_and(|r| r.kind == Kind::Art)
            };
            if near_art(i.checked_sub(1)) || near_art(Some(i + 1)) {
                raws[i].kind = Kind::Art;
            }
        }
    }

    let blank = |i: Option<usize>, raws: &[Raw]| {
        i.and_then(|i| raws.get(i))
            .is_none_or(|r| matches!(r.kind, Kind::Blank | Kind::Rule))
    };
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < raws.len() {
        let r = &raws[i];
        let t = r.text.trim_start();
        match r.kind {
            Kind::Blank => i += 1,
            Kind::Rule => {
                blocks.push(Block::Rule { src: r.at });
                i += 1;
            }
            Kind::Art => {
                let start = i;
                while i < raws.len() && raws[i].kind == Kind::Art {
                    i += 1;
                }
                let cut = raws[start..i].iter().map(|r| r.indent).min().unwrap_or(0);
                let lines = raws[start..i]
                    .iter()
                    .map(|r| (r.text[cut.min(r.text.len())..].to_string(), r.at + cut))
                    .collect();
                blocks.push(Block::Pre { lines });
            }
            Kind::Prose => {
                // a short line over an underline, or decorated, or in capitals on its own
                let underlined = raws.get(i + 1).is_some_and(|n| {
                    matches!(n.kind, Kind::Rule | Kind::Art)
                        && is_rule(n.text.trim())
                        && n.text.trim().chars().count() + 2 >= t.chars().count()
                });
                let heading = if let Some(inner) = decorated(t) {
                    Some((inner, 2))
                } else if underlined && t.chars().count() <= 60 {
                    Some((t, 2))
                } else if blank(i.checked_sub(1), &raws)
                    && blank(Some(i + 1), &raws)
                    && t.chars().count() <= 60
                    && t.chars().filter(|c| c.is_alphabetic()).count() >= 3
                    && !t.chars().any(char::is_lowercase)
                {
                    Some((t, 1))
                } else {
                    None
                };
                if let Some((inner, level)) = heading {
                    let off = t.find(inner).unwrap_or(0);
                    let mut words = Vec::new();
                    words_of(inner, r.at + r.indent + off, true, &mut words);
                    blocks.push(Block::Heading { words, level });
                    i += if underlined { 2 } else { 1 };
                    continue;
                }
                let indent = u8::try_from((r.indent - least) / 4).unwrap_or(4).min(4);
                let mut words = Vec::new();
                let block_indent = r.indent;
                let mark = item_mark(t);
                let body = mark.as_ref().map_or(0, |m| m.1);
                join_line(&mut words, &t[body..], r.at + r.indent + body);
                let mut short = r.text.chars().count() + 1 < wrap * 6 / 10;
                i += 1;
                while i < raws.len() && !short {
                    let n = &raws[i];
                    let nt = n.text.trim_start();
                    let goes_on = n.kind == Kind::Prose
                        && decorated(nt).is_none()
                        && item_mark(nt).is_none()
                        && if mark.is_some() {
                            n.indent > block_indent
                        } else {
                            n.indent <= block_indent + 1
                        };
                    if !goes_on {
                        break;
                    }
                    join_line(&mut words, nt, n.at + n.indent);
                    short = n.text.chars().count() + 1 < wrap * 6 / 10;
                    i += 1;
                }
                blocks.push(match mark {
                    Some((mark, _)) => Block::Item {
                        words,
                        indent,
                        mark,
                    },
                    None => Block::Para { words, indent },
                });
            }
        }
    }
    blocks
}

// ---------------------------------------------------------------- HTML

fn entity(name: &str) -> Option<char> {
    if let Some(num) = name.strip_prefix('#') {
        let n = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse().ok()?,
        };
        return char::from_u32(n);
    }
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "auml" => 'ä',
        "ouml" => 'ö',
        "uuml" => 'ü',
        "Auml" => 'Ä',
        "Ouml" => 'Ö',
        "Uuml" => 'Ü',
        "szlig" => 'ß',
        "eacute" => 'é',
        "egrave" => 'è',
        "agrave" => 'à',
        "aacute" => 'á',
        "ccedil" => 'ç',
        "ntilde" => 'ñ',
        "copy" => '©',
        "reg" => '®',
        "trade" => '™',
        "hellip" => '…',
        "mdash" => '—',
        "ndash" => '–',
        "lsquo" => '‘',
        "rsquo" => '’',
        "ldquo" => '“',
        "rdquo" => '”',
        "bdquo" => '„',
        "laquo" => '«',
        "raquo" => '»',
        "bull" => '•',
        "middot" => '·',
        "deg" => '°',
        "times" => '×',
        "rarr" => '→',
        "larr" => '←',
        "uarr" => '↑',
        "darr" => '↓',
        "frac12" => '½',
        "star" | "starf" => '★',
        _ => return None,
    })
}

/// `s` with its character references read.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let end = rest[1..].find(|c: char| c == ';' || c == '&' || c == '<' || c.is_whitespace());
        match end.filter(|&e| e <= 10).map(|e| e + 1) {
            Some(e) if rest[e..].starts_with(';') => match entity(&rest[1..e]) {
                Some(c) => {
                    out.push(c);
                    rest = &rest[e + 1..];
                }
                None => {
                    out.push('&');
                    rest = &rest[1..];
                }
            },
            _ => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

#[derive(Default)]
struct HtmlState {
    blocks: Vec<Block>,
    words: Vec<Word>,
    /// Whether the last text ended between words, so the next starts a new one.
    spaced: bool,
    bold: u32,
    skip: u32,
    pre: u32,
    pre_text: String,
    pre_at: usize,
    heading: Option<u8>,
    item: Option<String>,
    lists: Vec<Option<u32>>,
    quote: u8,
    cells: u32,
}

impl HtmlState {
    fn flush(&mut self) {
        if self.words.is_empty() {
            return;
        }
        let words = std::mem::take(&mut self.words);
        let indent = (self.lists.len().saturating_sub(1) as u8 + self.quote).min(4);
        let block = match (self.heading, self.item.take()) {
            (Some(level), _) => Block::Heading { words, level },
            (None, Some(mark)) => Block::Item {
                words,
                indent,
                mark,
            },
            (None, None) => Block::Para {
                words,
                indent: self.quote.min(4),
            },
        };
        self.blocks.push(block);
        self.spaced = true;
    }

    fn text(&mut self, raw: &str, at: usize) {
        if self.skip > 0 {
            return;
        }
        if self.pre > 0 {
            if self.pre_text.is_empty() {
                self.pre_at = at;
            }
            self.pre_text.push_str(&unescape(raw));
            return;
        }
        let text = unescape(raw).replace('\u{a0}', " ");
        let starts_spaced = text.starts_with(char::is_whitespace);
        let mut words = Vec::new();
        words_of(&text, at, self.bold > 0, &mut words);
        let mut words = words.into_iter();
        if !starts_spaced && !self.spaced {
            if let (Some(last), Some(first)) = (self.words.last_mut(), words.as_slice().first()) {
                last.text.push_str(&first.text);
                words.next();
            }
        }
        self.words.extend(words);
        if !text.trim().is_empty() || starts_spaced {
            self.spaced = text.ends_with(char::is_whitespace);
        }
    }

    fn end_pre(&mut self) {
        let text = std::mem::take(&mut self.pre_text);
        let text = text.strip_prefix('\n').unwrap_or(&text);
        let mut lines = Vec::new();
        let mut at = self.pre_at;
        for line in text.trim_end().split('\n') {
            lines.push((line.trim_end().to_string(), at));
            at += line.len() + 1;
        }
        if lines.iter().any(|l| !l.0.is_empty()) {
            self.blocks.push(Block::Pre { lines });
        }
    }

    fn tag(&mut self, name: &str, closing: bool, attrs: &str) {
        const SKIP: [&str; 7] = [
            "script", "style", "head", "template", "svg", "noscript", "select",
        ];
        if SKIP.contains(&name) {
            if closing {
                self.skip = self.skip.saturating_sub(1);
            } else {
                self.skip += 1;
            }
            return;
        }
        if self.skip > 0 {
            return;
        }
        if self.pre > 0 && name != "pre" {
            if name == "br" {
                self.pre_text.push('\n');
            }
            return;
        }
        match (name, closing) {
            ("b" | "strong", false) => self.bold += 1,
            ("b" | "strong", true) => self.bold = self.bold.saturating_sub(1),
            ("th", true) => self.bold = self.bold.saturating_sub(1),
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", false) => {
                self.flush();
                self.heading = Some(if name <= "h2" { 1 } else { 2 });
            }
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", true) => {
                self.flush();
                self.heading = None;
            }
            ("ul", false) => {
                self.flush();
                self.lists.push(None);
            }
            ("ol", false) => {
                self.flush();
                let start = attr(attrs, "start")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                self.lists.push(Some(start));
            }
            ("ul" | "ol", true) => {
                self.flush();
                self.lists.pop();
            }
            ("li", false) => {
                self.flush();
                let mark = match self.lists.last_mut() {
                    Some(Some(n)) => {
                        *n += 1;
                        format!("{}.", *n - 1)
                    }
                    _ => "•".into(),
                };
                self.item = Some(mark);
            }
            ("blockquote", false) => {
                self.flush();
                self.quote += 1;
            }
            ("blockquote", true) => {
                self.flush();
                self.quote = self.quote.saturating_sub(1);
            }
            ("pre", false) => {
                self.flush();
                self.pre += 1;
            }
            ("pre", true) => {
                self.pre = self.pre.saturating_sub(1);
                if self.pre == 0 {
                    self.end_pre();
                }
            }
            ("hr", _) => {
                self.flush();
                self.blocks.push(Block::Rule { src: self.pre_at });
            }
            ("tr", _) => {
                self.flush();
                self.cells = 0;
            }
            ("td" | "th", false) => {
                if self.cells > 0 && !self.words.is_empty() {
                    self.words.push(Word {
                        text: "│".into(),
                        bold: false,
                        src: self.words.last().map_or(0, |w| w.src),
                    });
                }
                self.cells += 1;
                self.spaced = true;
                if name == "th" {
                    self.bold += 1;
                }
            }
            ("br", _) => self.flush(),
            (
                "p" | "div" | "section" | "article" | "table" | "dt" | "dd" | "dl" | "tbody"
                | "thead" | "header" | "footer" | "main" | "center" | "caption" | "figure"
                | "figcaption" | "body" | "html" | "form",
                _,
            ) => self.flush(),
            ("img", _) => {
                if let Some(alt) = attr(attrs, "alt").filter(|a| !a.trim().is_empty()) {
                    self.text(&format!(" [{alt}] "), 0);
                }
            }
            _ => {}
        }
    }
}

/// The value of attribute `name` in a tag's attributes, if it has one.
fn attr(attrs: &str, name: &str) -> Option<String> {
    let lower = attrs.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = lower[from..].find(name) {
        let i = from + i;
        let before_ok = i == 0 || lower.as_bytes()[i - 1].is_ascii_whitespace();
        let rest = attrs[i + name.len()..].trim_start();
        if before_ok {
            if let Some(v) = rest.strip_prefix('=') {
                let v = v.trim_start();
                let value = match v.chars().next() {
                    Some(q @ ('"' | '\'')) => v[1..].split(q).next().unwrap_or(""),
                    _ => v
                        .split(|c: char| c.is_whitespace() || c == '>')
                        .next()
                        .unwrap_or(""),
                };
                return Some(unescape(value));
            }
        }
        from = i + name.len();
    }
    None
}

pub fn parse_html(src: &str) -> Vec<Block> {
    let mut st = HtmlState {
        spaced: true,
        ..Default::default()
    };
    let mut i = 0;
    let bytes = src.as_bytes();
    while i < src.len() {
        let Some(lt) = src[i..].find('<').map(|p| i + p) else {
            st.text(&src[i..], i);
            break;
        };
        if lt > i {
            st.text(&src[i..lt], i);
        }
        if src[lt..].starts_with("<!--") {
            i = src[lt..].find("-->").map_or(src.len(), |e| lt + e + 3);
            continue;
        }
        let Some(gt) = src[lt..].find('>').map(|p| lt + p) else {
            st.text(&src[lt..], lt);
            break;
        };
        let inner = &src[lt + 1..gt];
        let closing = inner.starts_with('/');
        let inner = inner.trim_start_matches('/');
        let end = inner
            .find(|c: char| c.is_whitespace() || c == '/')
            .unwrap_or(inner.len());
        let name = inner[..end].to_ascii_lowercase();
        if name.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            st.pre_at = if st.pre > 0 { st.pre_at } else { gt + 1 };
            st.tag(&name, closing, &inner[end..]);
            // the text of a script or a style is not markup: skip to its end
            if !closing && (name == "script" || name == "style") {
                let close = format!("</{name}");
                let lower = src[gt..].to_ascii_lowercase();
                i = lower.find(&close).map_or(src.len(), |p| gt + p);
                continue;
            }
        } else if bytes
            .get(lt + 1)
            .is_some_and(|b| !b.is_ascii_alphabetic() && *b != b'/' && *b != b'!')
        {
            // a lone "<" in the text
            st.text(&src[lt..gt + 1], lt);
        }
        i = gt + 1;
    }
    if st.pre > 0 {
        st.end_pre();
    }
    st.flush();
    st.blocks
}

// ---------------------------------------------------------------- fonts

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Face {
    Sans,
    Mono,
}

fn font(face: Face) -> Option<&'static Font> {
    static SANS: OnceLock<Option<Font>> = OnceLock::new();
    static MONO: OnceLock<Option<Font>> = OnceLock::new();
    let (cell, bytes) = match face {
        Face::Sans => (&SANS, SANS_TTF),
        Face::Mono => (&MONO, MONO_TTF),
    };
    cell.get_or_init(|| Font::from_bytes(bytes, FontSettings::default()).ok())
        .as_ref()
}

/// How far each character moves the pen, per pixel of size, remembered once measured.
#[derive(Default)]
pub struct Advances {
    known: HashMap<(Face, char), f32>,
}

impl Advances {
    pub fn of(&mut self, face: Face, ch: char) -> f32 {
        *self.known.entry((face, ch)).or_insert_with(|| {
            font(face).map_or(0.6, |f| f.metrics(ch, 100.0).advance_width / 100.0)
        })
    }

    pub fn width(&mut self, face: Face, text: &str, px: f32) -> f32 {
        text.chars().map(|c| self.of(face, c)).sum::<f32>() * px
    }
}

// ---------------------------------------------------------------- layout

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ink {
    Body,
    Head,
    Dim,
    Mono,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub x: f32,
    pub text: String,
    pub face: Face,
    pub px: f32,
    pub bold: bool,
    pub ink: Ink,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Its top, down from the guide's top, and how tall it is.
    pub y: f32,
    pub h: f32,
    /// Where its words sit, down from its top.
    pub base: f32,
    pub runs: Vec<Run>,
    pub rule: bool,
    /// A picture shown at its left edge, a square `entry_icon_px` across, its top at the line's.
    pub icon: Option<std::sync::Arc<Vec<u8>>>,
    /// Where its first word is in the guide's text.
    pub at: usize,
}

/// The guide laid out across `width` at one size.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Laid {
    pub lines: Vec<Line>,
    pub height: f32,
    pub px: f32,
}

impl Laid {
    /// How far down the line the guide's text at `src` is on starts: the first line at or past it.
    pub fn y_of(&self, src: usize) -> f32 {
        let i = self.lines.partition_point(|l| l.at < src);
        let i = i.min(self.lines.len().saturating_sub(1));
        // a line before it may hold it, part way along
        let i = if self.lines.get(i).is_some_and(|l| l.at > src) {
            i.saturating_sub(1)
        } else {
            i
        };
        self.lines.get(i).map_or(0.0, |l| l.y)
    }

    /// Where in the guide's text the first line showing at `top` starts.
    pub fn src_at(&self, top: f32) -> usize {
        let i = self.lines.partition_point(|l| l.y + l.h <= top + 0.5);
        self.lines.get(i).map_or(0, |l| l.at)
    }
}

fn ascent(face: Face, px: f32) -> f32 {
    font(face)
        .and_then(|f| f.horizontal_line_metrics(px))
        .map_or(px * 0.8, |m| m.ascent)
}

/// Sets `words` across `width` from `left`, as many on each line as fit, at `px` in `face`.
#[allow(clippy::too_many_arguments)]
fn flow(
    adv: &mut Advances,
    out: &mut Laid,
    words: &[Word],
    left: f32,
    width: f32,
    px: f32,
    ink: Ink,
    mark: Option<&str>,
    force_bold: bool,
) {
    let face = Face::Sans;
    let space = adv.of(face, ' ') * px;
    let line_h = (px * 1.4).round();
    let base = ascent(face, px).round() + ((line_h - px * 1.17) / 2.0).max(0.0).round();
    let mut line: Vec<Run> = Vec::new();
    let mut at = words.first().map_or(0, |w| w.src);
    let mut x = left;
    let push = |out: &mut Laid, runs: Vec<Run>, at: usize| {
        out.lines.push(Line {
            y: out.height,
            h: line_h,
            base,
            runs,
            rule: false,
            icon: None,
            at,
        });
        out.height += line_h;
    };
    if let Some(mark) = mark {
        let w = adv.width(face, mark, px);
        line.push(Run {
            x: left - w - space,
            text: mark.into(),
            face,
            px,
            bold: false,
            ink: Ink::Dim,
        });
    }
    for word in words {
        let bold = word.bold || force_bold;
        let w = adv.width(face, &word.text, px) + if bold { 1.0 } else { 0.0 };
        let started = line.iter().any(|r| r.ink != Ink::Dim || mark.is_none());
        if started && x + w > left + width {
            push(out, std::mem::take(&mut line), at);
            x = left;
            at = word.src;
        }
        if !line.iter().any(|r| r.ink != Ink::Dim || mark.is_none()) {
            at = word.src;
        }
        // a word wider than the screen is cut over lines
        if w > width {
            let mut piece = String::new();
            for ch in word.text.chars() {
                let cw = adv.of(face, ch) * px;
                if x + adv.width(face, &piece, px) + cw > left + width && !piece.is_empty() {
                    line.push(Run {
                        x,
                        text: std::mem::take(&mut piece),
                        face,
                        px,
                        bold,
                        ink,
                    });
                    push(out, std::mem::take(&mut line), at);
                    x = left;
                }
                piece.push(ch);
            }
            let pw = adv.width(face, &piece, px);
            line.push(Run {
                x,
                text: piece,
                face,
                px,
                bold,
                ink,
            });
            x += pw + space;
            continue;
        }
        line.push(Run {
            x,
            text: word.text.clone(),
            face,
            px,
            bold,
            ink,
        });
        x += w + space;
    }
    if !line.is_empty() {
        push(out, line, at);
    }
}

/// The guide's `blocks` laid out `width` across at size `px`.
pub fn lay_out(adv: &mut Advances, blocks: &[Block], width: f32, px: f32) -> Laid {
    let mut out = Laid {
        px,
        ..Default::default()
    };
    let gap = (px * 0.55).round();
    let step = (px * 1.3).round();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            out.height += gap;
        }
        match block {
            Block::Para { words, indent } => {
                let left = f32::from(*indent) * step;
                flow(
                    adv,
                    &mut out,
                    words,
                    left,
                    width - left,
                    px,
                    Ink::Body,
                    None,
                    false,
                );
            }
            Block::Heading { words, level } => {
                let size = if *level <= 1 { px * 1.25 } else { px * 1.1 }.round();
                if i > 0 {
                    out.height += gap;
                }
                flow(
                    adv,
                    &mut out,
                    words,
                    0.0,
                    width,
                    size,
                    Ink::Head,
                    None,
                    true,
                );
            }
            Block::Item {
                words,
                indent,
                mark,
            } => {
                let left = (f32::from(*indent) + 1.0) * step;
                flow(
                    adv,
                    &mut out,
                    words,
                    left,
                    width - left,
                    px,
                    Ink::Body,
                    Some(mark),
                    false,
                );
            }
            Block::Pre { lines } => {
                let cols = lines.iter().map(|l| l.0.chars().count()).max().unwrap_or(0);
                let unit = adv.of(Face::Mono, 'M');
                let fit = if cols == 0 {
                    px
                } else {
                    width / (cols as f32 * unit)
                };
                let size = (px.min(fit) * 2.0).floor() / 2.0;
                let size = size.max(MONO_LEAST);
                let h = (size * 1.2).round().max(1.0);
                let base = ascent(Face::Mono, size).round();
                for (text, at) in lines {
                    out.lines.push(Line {
                        y: out.height,
                        h,
                        base,
                        runs: vec![Run {
                            x: 0.0,
                            text: text.clone(),
                            face: Face::Mono,
                            px: size,
                            bold: false,
                            ink: Ink::Mono,
                        }],
                        rule: false,
                        icon: None,
                        at: *at,
                    });
                    out.height += h;
                }
            }
            Block::Entry {
                icon,
                title,
                detail,
                locked,
            } => {
                let size = entry_icon_px(px);
                let left = if icon.is_some() {
                    size + (px * 0.6).round()
                } else {
                    0.0
                };
                let (first, top) = (out.lines.len(), out.height);
                let (head, body) = if *locked {
                    (Ink::Dim, Ink::Dim)
                } else {
                    (Ink::Head, Ink::Body)
                };
                flow(
                    adv,
                    &mut out,
                    title,
                    left,
                    width - left,
                    px,
                    head,
                    None,
                    true,
                );
                if !detail.is_empty() {
                    flow(
                        adv,
                        &mut out,
                        detail,
                        left,
                        width - left,
                        px,
                        body,
                        None,
                        false,
                    );
                }
                // the picture is as tall as it is, whatever the words are
                if out.height - top < size {
                    out.height = top + size;
                }
                if let (Some(icon), Some(line)) = (icon, out.lines.get_mut(first)) {
                    line.icon = Some(icon.clone());
                }
            }
            Block::Rule { src } => {
                out.lines.push(Line {
                    y: out.height,
                    h: px,
                    base: 0.0,
                    runs: Vec::new(),
                    rule: true,
                    icon: None,
                    at: *src,
                });
                out.height += px;
            }
        }
    }
    out
}

// ---------------------------------------------------------------- drawing

/// Each glyph drawn once at each size it is read at.
#[derive(Default)]
pub struct Glyphs {
    known: HashMap<(Face, u32, char), (Metrics, Vec<u8>)>,
}

impl Glyphs {
    fn get(&mut self, face: Face, px: f32, ch: char) -> Option<&(Metrics, Vec<u8>)> {
        let f = font(face)?;
        Some(
            self.known
                .entry((face, px.to_bits(), ch))
                .or_insert_with(|| f.rasterize(ch, px)),
        )
    }
}

fn blend(px: &mut [u8], a: u32, colour: [u8; 3]) {
    for c in 0..3 {
        px[c] = ((colour[c] as u32 * a + px[c] as u32 * (255 - a)) / 255) as u8;
    }
}

struct Canvas<'a> {
    rgba: &'a mut [u8],
    w: u32,
    /// The rows that may be drawn on, top inclusive and bottom not.
    clip: (i32, i32),
}

impl Canvas<'_> {
    fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, colour: [u8; 3]) {
        for row in y.max(self.clip.0)..(y + h).min(self.clip.1) {
            for col in x.max(0)..(x + w).min(self.w as i32) {
                let i = (row as usize * self.w as usize + col as usize) * 4;
                self.rgba[i..i + 3].copy_from_slice(&colour);
                self.rgba[i + 3] = 255;
            }
        }
    }

    /// A square picture of 64 by 64 RGBA pixels, shown `size` across with its top left at
    /// (`x`, `y`), each pixel the nearest of the picture's, laid over what is there.
    fn picture(&mut self, x: i32, y: i32, size: i32, rgba: &[u8]) {
        const SIDE: i32 = 64;
        if size <= 0 || rgba.len() < (SIDE * SIDE * 4) as usize {
            return;
        }
        for row in 0..size {
            let py = y + row;
            if py < self.clip.0 || py >= self.clip.1 {
                continue;
            }
            for col in 0..size {
                let px = x + col;
                if px < 0 || px >= self.w as i32 {
                    continue;
                }
                let (sx, sy) = (col * SIDE / size, row * SIDE / size);
                let s = &rgba[((sy * SIDE + sx) * 4) as usize..][..4];
                let i = (py as usize * self.w as usize + px as usize) * 4;
                blend(&mut self.rgba[i..i + 4], s[3] as u32, [s[0], s[1], s[2]]);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn text(
        &mut self,
        glyphs: &mut Glyphs,
        face: Face,
        px: f32,
        text: &str,
        x: f32,
        baseline: i32,
        colour: [u8; 3],
        bold: bool,
    ) -> f32 {
        let mut pen = x;
        for ch in text.chars() {
            let Some((m, cov)) = glyphs.get(face, px, ch) else {
                continue;
            };
            let gx = pen.round() as i32 + m.xmin;
            let gy = baseline - m.ymin - m.height as i32;
            for row in 0..m.height as i32 {
                let y = gy + row;
                if y < self.clip.0 || y >= self.clip.1 {
                    continue;
                }
                for col in 0..m.width as i32 {
                    let a = cov[(row * m.width as i32 + col) as usize] as u32;
                    if a == 0 {
                        continue;
                    }
                    for dx in 0..if bold { 2 } else { 1 } {
                        let x = gx + col + dx;
                        if x < 0 || x >= self.w as i32 {
                            continue;
                        }
                        let i = (y as usize * self.w as usize + x as usize) * 4;
                        blend(&mut self.rgba[i..i + 4], a, colour);
                    }
                }
            }
            pen += m.advance_width;
        }
        pen - x
    }
}

/// What a page of the guide shows besides the guide: its name across the top and the keys
/// across the bottom.
pub struct Page<'a> {
    pub title: &'a str,
    pub keys: &'a str,
    pub ink: GuideInk,
    /// The tabs across the top, left, and which one is open: none for a bare page.
    pub tabs: &'a [&'a str],
    pub tab: usize,
}

/// The guide's text area on a `w` by `h` screen, `[x, y, w, h]` in screen pixels.
pub fn text_area(w: u32, h: u32) -> [u32; 4] {
    let margin = (w / 40).max(8);
    [
        margin,
        GUIDE_HEAD_H + 6,
        w.saturating_sub(2 * margin),
        h.saturating_sub(GUIDE_HEAD_H + GUIDE_FOOT_H + 12),
    ]
}

/// The furthest down the guide its top can be scrolled on a screen `h` tall.
pub fn max_top(laid: &Laid, h: u32) -> f32 {
    let area = text_area(1, h)[3] as f32;
    (laid.height - area).max(0.0)
}

/// The page of the guide whose top is `top` pixels down it, drawn on a `w` by `h` screen.
pub fn render_page(
    glyphs: &mut Glyphs,
    laid: &Laid,
    top: f32,
    page: &Page,
    w: u32,
    h: u32,
) -> Vec<u8> {
    let ink = page.ink;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let [ax, ay, aw, ah] = text_area(w, h);
    let mut c = Canvas {
        rgba: &mut rgba,
        w,
        clip: (0, h as i32),
    };
    c.fill(0, 0, w as i32, h as i32, ink.bg);
    c.fill(0, 0, w as i32, GUIDE_HEAD_H as i32, ink.band);
    c.fill(
        0,
        (h - GUIDE_FOOT_H) as i32,
        w as i32,
        GUIDE_FOOT_H as i32,
        ink.band,
    );

    let band_px = 14.0;
    let band_base = |top: u32, band: u32| (top + band / 2) as i32 + (band_px * 0.36) as i32;
    let max = max_top(laid, h);
    let percent = if max <= 0.0 {
        100
    } else {
        ((top / max) * 100.0).round() as u32
    };
    let right = format!("{percent}%");
    let mut adv = Advances::default();
    let rw = adv.width(Face::Sans, &right, band_px);
    c.text(
        glyphs,
        Face::Sans,
        band_px,
        &right,
        w as f32 - ax as f32 - rw,
        band_base(0, GUIDE_HEAD_H),
        ink.dim,
        false,
    );
    // the tabs, the open one lit; the guide's own name goes after them when there is room
    let mut x = ax as f32;
    for (i, tab) in page.tabs.iter().enumerate() {
        let lit = i == page.tab;
        c.text(
            glyphs,
            Face::Sans,
            band_px,
            tab,
            x,
            band_base(0, GUIDE_HEAD_H),
            if lit { ink.head } else { ink.dim },
            lit,
        );
        x += adv.width(Face::Sans, tab, band_px) + 18.0;
    }
    let mut title = page.title.to_string();
    let room = w as f32 - x - 2.0 * ax as f32 - rw;
    while !title.is_empty() && adv.width(Face::Sans, &title, band_px) > room {
        title.pop();
    }
    c.text(
        glyphs,
        Face::Sans,
        band_px,
        &title,
        x,
        band_base(0, GUIDE_HEAD_H),
        if page.tabs.is_empty() {
            ink.head
        } else {
            ink.dim
        },
        page.tabs.is_empty(),
    );
    let kw = adv.width(Face::Sans, page.keys, 12.0);
    c.text(
        glyphs,
        Face::Sans,
        12.0,
        page.keys,
        ((w as f32 - kw) / 2.0).max(4.0),
        (h - GUIDE_FOOT_H / 2) as i32 + 4,
        ink.dim,
        false,
    );

    c.clip = (ay as i32, (ay + ah) as i32);
    let first = laid.lines.partition_point(|l| l.y + l.h <= top);
    for line in &laid.lines[first..] {
        let y = ay as f32 + line.y - top;
        if y >= (ay + ah) as f32 {
            break;
        }
        if line.rule {
            let mid = (y + line.h / 2.0) as i32;
            c.fill(ax as i32, mid, aw as i32, 1, ink.dim);
            continue;
        }
        if let Some(icon) = &line.icon {
            c.picture(
                ax as i32,
                y.round() as i32,
                entry_icon_px(laid.px) as i32,
                icon,
            );
        }
        let baseline = (y + line.base).round() as i32;
        for run in &line.runs {
            let colour = match run.ink {
                Ink::Body => ink.body,
                Ink::Head => ink.head,
                Ink::Dim => ink.dim,
                Ink::Mono => ink.mono,
            };
            c.text(
                glyphs,
                run.face,
                run.px,
                &run.text,
                ax as f32 + run.x,
                baseline,
                colour,
                run.bold,
            );
        }
    }
    // how far in, as a thin bar down the right edge
    if laid.height > ah as f32 {
        let track = ah as f32;
        let thumb = (track * track / laid.height).max(12.0);
        let ty = ay as f32 + (track - thumb) * if max > 0.0 { top / max } else { 0.0 };
        c.fill((w - 3) as i32, ty as i32, 2, thumb as i32, ink.dim);
    }
    rgba
}
