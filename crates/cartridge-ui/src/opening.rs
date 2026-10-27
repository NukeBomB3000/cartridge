//! The opening's lettering, after Mega Man X5's NOM ENGINEER WORK SYSTEM boot screen: Pixel
//! Code set on its grid, each of its pixels drawn three screen pixels wide and two tall, as
//! wide as X5's letters, drawn bold by a screen pixel to stand as heavy, and shaded a row of
//! the face at a time in X5's mint, pale through the middle.

use fontdue::Font;

use crate::kit::pixel::LETTER_PX;
use crate::plate::UndoFace;
use crate::text::label_font;

/// How far apart two letters stand, in screen pixels.
pub const OPENING_ADVANCE: u32 = (CELL as u32) * SX;

/// How far apart two lines stand, in screen pixels: the face's whole height, so a tail stops
/// where the next line's capitals start.
pub const OPENING_PITCH: u32 = 9 * SY;

/// How far above its capitals a line's face starts: the face's row above them, which a quote
/// or a bracket can reach into.
pub const OPENING_RISE: u32 = SY;

/// How many screen pixels each of the face's pixels takes, across and down.
const SX: u32 = 3;
const SY: u32 = 2;

/// Each letter's cell on the face's grid.
const CELL: usize = 6;

/// The face's rows a line is drawn in: one above the capitals, seven of them, two of tail.
const ROWS: usize = 10;

/// The row the capitals stand on.
const BASELINE: i32 = 8;

/// X5's mint for each of those rows: pale through the middle of the capitals, deeper at their
/// foot and deeper still in the tails.
const BANDS: [[u8; 3]; ROWS] = [
    [161, 228, 208],
    [161, 228, 208],
    [161, 228, 208],
    [214, 238, 232],
    [214, 238, 232],
    [176, 230, 214],
    [176, 230, 214],
    [140, 204, 186],
    [116, 178, 162],
    [116, 178, 162],
];

/// `text` as the opening draws it, one line, on a clear face: `OPENING_ADVANCE` a letter and
/// one screen pixel more for the bold, `OPENING_RISE` above the capitals to the foot of the
/// tails.
pub fn opening_line(text: &str) -> UndoFace {
    let ink = match label_font() {
        Some(font) => grid_ink(font, text),
        None => vec![[false; ROWS]; 0],
    };
    let n = text.chars().count();
    let w = n as u32 * OPENING_ADVANCE + 1;
    let h = ROWS as u32 * SY;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for (col, rows) in ink.iter().enumerate() {
        for (row, &inked) in rows.iter().enumerate() {
            if !inked {
                continue;
            }
            let [r, g, b] = BANDS[row];
            for y in row as u32 * SY..(row as u32 + 1) * SY {
                // a face pixel and the bold's screen pixel to its right
                for x in col as u32 * SX..=(col as u32 + 1) * SX {
                    let i = ((y * w + x) * 4) as usize;
                    rgba[i..i + 4].copy_from_slice(&[r, g, b, 255]);
                }
            }
        }
    }
    UndoFace { rgba, w, h }
}

/// Which of the face's pixels `text` inks, a column at a time, each letter in its own cell.
fn grid_ink(font: &Font, text: &str) -> Vec<[bool; ROWS]> {
    let mut ink = vec![[false; ROWS]; text.chars().count() * CELL];
    for (i, ch) in text.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        // at one face pixel to a pixel of `ink`, which then sizes it up
        let (m, cov) = font.rasterize(ch, LETTER_PX);
        let top = BASELINE - (m.height as i32 + m.ymin) + drop(ch);
        for r in 0..m.height {
            for c in 0..m.width {
                let (x, y) = ((i * CELL) as i32 + m.xmin + c as i32, top + r as i32);
                if cov[r * m.width + c] < 128 || x < 0 || y < 0 || y >= ROWS as i32 {
                    continue;
                }
                if let Some(col) = ink.get_mut(x as usize) {
                    col[y as usize] = true;
                }
            }
        }
    }
    ink
}

/// How many of the face's rows a letter is let down: Pixel Code's star sits up by the
/// capitals' tops, X5's in the middle of the line, as a masked code is typed.
fn drop(ch: char) -> i32 {
    if ch == '*' {
        2
    } else {
        0
    }
}
