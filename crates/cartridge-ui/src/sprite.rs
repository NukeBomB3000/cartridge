//! Pixel art drawn on the same grid as the words beside it: one of a sprite's pixels is one of
//! Pixel Code's at the size the bars set their words in, so an icon next to a word looks cut
//! from the same face.

use crate::plate::UndoFace;

/// Screen pixels to a side of one of a sprite's, the size of one of Pixel Code's pixels at 18 px.
pub const SPRITE_PX: u32 = 2;

/// Every sprite is drawn in a square of this many of its pixels, the height of the face's
/// capitals and figures, so icons and words stand exactly as tall as each other.
pub const CELL: u32 = 7;

/// A cell's side on the screen.
pub const CELL_PX: f32 = (CELL * SPRITE_PX) as f32;

/// A sprite, row by row from the top: `#` is inked, anything else is clear.
pub type Rows = [&'static str; CELL as usize];

/// `rows` in `ink`, a texel to each of its pixels: a face at art resolution, to upload NEAREST
/// and draw at `SPRITE_PX` screen pixels to a texel (`kit::pixel::quad`).
pub fn sprite_face(rows: &[&str], ink: [u8; 3]) -> UndoFace {
    let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as u32;
    let h = rows.len() as u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for (y, row) in rows.iter().enumerate() {
        for (x, key) in row.bytes().enumerate() {
            if key == b'#' {
                let i = (y * w as usize + x) * 4;
                rgba[i..i + 4].copy_from_slice(&[ink[0], ink[1], ink[2], 255]);
            }
        }
    }
    UndoFace { rgba, w, h }
}

/// `face` cut down to the box around its ink, so it can be placed by what is drawn and not by
/// the air its renderer left round it.
pub fn cropped(face: UndoFace) -> UndoFace {
    let inked = |x: u32, y: u32| face.rgba[((y * face.w + x) * 4 + 3) as usize] > 0;
    let rows: Vec<u32> = (0..face.h)
        .filter(|&y| (0..face.w).any(|x| inked(x, y)))
        .collect();
    let cols: Vec<u32> = (0..face.w)
        .filter(|&x| (0..face.h).any(|y| inked(x, y)))
        .collect();
    let (Some(&y0), Some(&y1), Some(&x0), Some(&x1)) =
        (rows.first(), rows.last(), cols.first(), cols.last())
    else {
        return face;
    };
    let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in y0..=y1 {
        let start = ((y * face.w + x0) * 4) as usize;
        rgba.extend_from_slice(&face.rgba[start..start + (w * 4) as usize]);
    }
    UndoFace { rgba, w, h }
}
