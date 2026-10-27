use std::sync::OnceLock;

use fontdue::{Font, FontSettings};

/// Every word cartridge prints is set in Pixel Code, a pixel face drawn on a grid of nine to the em.
const LABEL_TTF: &[u8] = include_bytes!("../assets/PixelCode-Regular.ttf");

/// The size at which one of the face's pixels is an art pixel, two of the screen's: the grid
/// the UI is drawn on (`SPRITE_PX`). Text is only ever set at a whole multiple of it, so every
/// stroke lands on whole art pixels and stays sharp, never half of one (9 px) or one and a half
/// (27 px).
pub const FONT_GRID: f32 = 18.0;

/// `px` moved onto the face's grid: the nearest whole multiple, and never less than one.
pub fn on_grid(px: f32) -> f32 {
    (px / FONT_GRID).round().max(1.0) * FONT_GRID
}

/// The largest size on the grid no larger than `px`, and never less than one step.
fn grid_floor(px: f32) -> f32 {
    (px / FONT_GRID).floor().max(1.0) * FONT_GRID
}

pub struct Layout {
    pub lines: Vec<String>,
    pub px: f32,
    pub tracking: f32,
}

pub fn label_font() -> Option<&'static Font> {
    static FONT: OnceLock<Option<Font>> = OnceLock::new();
    FONT.get_or_init(|| Font::from_bytes(LABEL_TTF, FontSettings::default()).ok())
        .as_ref()
}

pub fn fit(
    font: &Font,
    text: &str,
    max_w: f32,
    max_lines: usize,
    max_px: f32,
    min_px: f32,
) -> Layout {
    fit_box(font, text, max_w, f32::INFINITY, max_lines, max_px, min_px)
}

#[allow(clippy::too_many_arguments)]
pub fn fit_box(
    font: &Font,
    text: &str,
    max_w: f32,
    max_h: f32,
    max_lines: usize,
    max_px: f32,
    min_px: f32,
) -> Layout {
    let upper = text.to_uppercase();
    let min_px = grid_floor(min_px);
    let mut px = grid_floor(max_px).max(min_px);
    while px > min_px {
        let tracking = tracking_for(px);
        let lines = wrap(font, &upper, px, tracking, max_w, false);
        if lines.len() <= max_lines
            && lines
                .iter()
                .all(|l| line_width(font, l, px, tracking) <= max_w)
            && block_height(font, px, lines.len()) <= max_h
        {
            return Layout {
                lines,
                px,
                tracking,
            };
        }
        px -= FONT_GRID;
    }
    let tracking = tracking_for(min_px);
    Layout {
        lines: wrap(font, &upper, min_px, tracking, max_w, true)
            .into_iter()
            .take(max_lines)
            .collect(),
        px: min_px,
        tracking,
    }
}

pub fn draw_centred(dst: &mut [u8], dst_w: u32, dst_h: u32, layout: &Layout, colour: [u8; 3]) {
    for (i, a) in coverage(dst_w, dst_h, layout).into_iter().enumerate() {
        if a > 0 {
            blend(&mut dst[i * 4..i * 4 + 4], a as u32, colour);
        }
    }
}

pub fn coverage(dst_w: u32, dst_h: u32, layout: &Layout) -> Vec<u8> {
    match label_font() {
        Some(font) => coverage_in(font, dst_w, dst_h, layout),
        None => vec![0u8; (dst_w * dst_h) as usize],
    }
}

/// The ink of `layout` set in `font`, centred in a `dst_w` by `dst_h` box. The size is moved
/// onto the face's grid and every line starts on a whole pixel, so each of the face's pixels
/// is a whole number of the screen's, and is either inked or not.
pub fn coverage_in(font: &Font, dst_w: u32, dst_h: u32, layout: &Layout) -> Vec<u8> {
    let mut out = vec![0u8; (dst_w * dst_h) as usize];
    let px = on_grid(layout.px);
    let Some(vm) = font.horizontal_line_metrics(px) else {
        return out;
    };
    let line_h = vm.new_line_size.round();
    let block_h = line_h * layout.lines.len() as f32;
    let mut baseline = ((dst_h as f32 - block_h) / 2.0 + vm.ascent).round();

    for line in &layout.lines {
        let mut pen = ((dst_w as f32 - line_width(font, line, px, layout.tracking)) / 2.0).round();
        for ch in printed(font, line).chars() {
            let (m, cov) = font.rasterize(ch, px);
            let cov: Vec<u8> = cov
                .iter()
                .map(|&a| if a >= 128 { 255 } else { 0 })
                .collect();
            stamp(
                &mut out,
                dst_w,
                dst_h,
                (pen + m.xmin as f32).round() as i32,
                (baseline - (m.height as f32 + m.ymin as f32)).round() as i32,
                &cov,
                m.width as u32,
                m.height as u32,
            );
            pen += m.advance_width + layout.tracking;
        }
        baseline += line_h;
    }
    out
}

/// How far below the top of a `dst_h` box `coverage_in` sets the top of a one-line word's
/// capitals and figures at `px`: where to hang the box to line them up with something else.
pub fn cap_top(font: &Font, dst_h: u32, px: f32) -> f32 {
    let px = on_grid(px);
    let Some(vm) = font.horizontal_line_metrics(px) else {
        return 0.0;
    };
    let baseline = ((dst_h as f32 - vm.new_line_size.round()) / 2.0 + vm.ascent).round();
    let (m, cov) = font.rasterize('0', px);
    // the glyph's box can carry a row its ink never reaches, which `coverage_in` leaves clear
    let air = (0..m.height)
        .take_while(|&r| cov[r * m.width..(r + 1) * m.width].iter().all(|&a| a < 128))
        .count();
    baseline - (m.height as f32 + m.ymin as f32) + air as f32
}

fn block_height(font: &Font, px: f32, lines: usize) -> f32 {
    match font.horizontal_line_metrics(on_grid(px)) {
        Some(vm) => vm.new_line_size.round() * lines as f32,
        None => 0.0,
    }
}

/// What `line` is printed as in `font`: a character the face has no glyph for is spelled out
/// the way a typewriter would, (C) for ©, or left out when it is only a hint, like a soft
/// hyphen.
pub fn printed(font: &Font, line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    for c in line.chars() {
        if font.lookup_glyph_index(c) != 0 || c == ' ' {
            out.push(c);
            continue;
        }
        match c {
            '\u{a9}' => out.push_str("(C)"),
            '\u{ae}' => out.push_str("(R)"),
            '\u{ad}' => {}
            _ => out.push('?'),
        }
    }
    out
}

/// How wide `line` is, measured at the grid size it is drawn at.
pub fn line_width(font: &Font, line: &str, px: f32, tracking: f32) -> f32 {
    let px = on_grid(px);
    let mut w = 0.0;
    let mut n = 0;
    for ch in printed(font, line).chars() {
        w += font.metrics(ch, px).advance_width;
        n += 1;
    }
    w + tracking * (n as f32 - 1.0).max(0.0)
}

/// The space added between capitals. The face already leaves a pixel between its letters,
/// which is all the air capitals need at its sizes.
fn tracking_for(_px: f32) -> f32 {
    0.0
}

fn wrap(
    font: &Font,
    text: &str,
    px: f32,
    tracking: f32,
    max_w: f32,
    break_words: bool,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let parts = if break_words {
            break_word(font, word, px, tracking, max_w)
        } else {
            vec![word.to_string()]
        };
        for part in parts {
            let joined = if line.is_empty() {
                part.clone()
            } else {
                format!("{line} {part}")
            };
            if line.is_empty() || line_width(font, &joined, px, tracking) <= max_w {
                line = joined;
            } else {
                lines.push(std::mem::take(&mut line));
                line = part;
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn break_word(font: &Font, word: &str, px: f32, tracking: f32, max_w: f32) -> Vec<String> {
    if line_width(font, word, px, tracking) <= max_w {
        return vec![word.to_string()];
    }
    let mut parts = Vec::new();
    let mut part = String::new();
    for ch in word.chars() {
        part.push(ch);
        if line_width(font, &part, px, tracking) > max_w && part.chars().count() > 1 {
            part.pop();
            parts.push(std::mem::take(&mut part));
            part.push(ch);
        }
    }
    if !part.is_empty() {
        parts.push(part);
    }
    parts
}

#[allow(clippy::too_many_arguments)]
fn stamp(dst: &mut [u8], dst_w: u32, dst_h: u32, x: i32, y: i32, cov: &[u8], gw: u32, gh: u32) {
    for gy in 0..gh {
        let dy = y + gy as i32;
        if dy < 0 || dy >= dst_h as i32 {
            continue;
        }
        for gx in 0..gw {
            let dx = x + gx as i32;
            if dx < 0 || dx >= dst_w as i32 {
                continue;
            }
            let i = (dy as u32 * dst_w + dx as u32) as usize;
            dst[i] = dst[i].max(cov[(gy * gw + gx) as usize]);
        }
    }
}

fn blend(px: &mut [u8], a: u32, colour: [u8; 3]) {
    let under = px[3] as u32 * (255 - a) / 255;
    let out = a + under;
    for c in 0..3 {
        px[c] = ((colour[c] as u32 * a + px[c] as u32 * under + out / 2) / out) as u8;
    }
    px[3] = out as u8;
}
