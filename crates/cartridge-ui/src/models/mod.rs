//! The shelf's models beyond the Game Boy line: each one's outline, thickness, face and how big
//! it stands, kept in its own file so each can be drawn on its own.

pub mod book;
pub mod disc;
pub mod hucard;
pub mod md;
pub mod nds;
pub mod nes;
pub mod polaroid;
pub mod sega8;
pub mod snes;
pub mod tape;
pub mod vhs;

/// A rectangle `w` by `h` with its corners rounded `r`, in drawing units, all the way round
/// clockwise from the top left, and the size of the drawing: each corner in three cuts.
pub(crate) fn rounded(w: f32, h: f32, r: f32) -> (Vec<(f32, f32)>, (f32, f32)) {
    let corners = [
        (r, r, 180.0f32),
        (w - r, r, 270.0),
        (w - r, h - r, 0.0),
        (r, h - r, 90.0),
    ];
    let mut points = Vec::new();
    for (cx, cy, from) in corners {
        for k in 0..=3 {
            let a = (from + 30.0 * k as f32).to_radians();
            points.push((cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    (points, (w, h))
}
