use crate::canvas::{out_h, out_w};

/// The line and the dot a picture closes to are an art pixel thick.
const LINE_PX: f32 = 2.0;

// A picture comes on as a tube TV's does, `t` going from 0 to 1: a dot of light in the middle
// (to DOT), the dot drawn out into a line across (to LINE), the line opening out into the
// picture (to OPEN), white all the while and fading to itself once it is open (to 1). Going
// off is the same backwards: the picture flares, folds into a line, the line into a dot, and
// the dot fades.
const DOT: f32 = 0.1;
const LINE: f32 = 0.35;
const OPEN: f32 = 0.7;
/// How white the picture is once it is open, fading from there to none.
const AFTERGLOW: f32 = 0.35;
/// How much brighter than itself the picture is while it glows.
const OVERSHOOT: f32 = 0.3;

fn ease_out(u: f32) -> f32 {
    let k = 1.0 - u.clamp(0.0, 1.0);
    1.0 - k * k
}

fn span(t: f32, from: f32, to: f32) -> f32 {
    ((t - from) / (to - from)).clamp(0.0, 1.0)
}

/// How tall the picture is at power `t`, as a share of its full height.
pub fn screen_scale(t: f32) -> f32 {
    let line = LINE_PX / out_h() as f32;
    line + (1.0 - line) * ease_out(span(t, LINE, OPEN))
}

/// How wide the picture is at power `t`, as a share of its full width.
pub fn screen_width(t: f32) -> f32 {
    let dot = LINE_PX / out_w() as f32;
    dot + (1.0 - dot) * ease_out(span(t, DOT, LINE))
}

/// How much of the picture is washed white at power `t`: all of it until the line opens, a
/// little once open, none at full power.
pub fn screen_glow(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < LINE {
        1.0
    } else if t < OPEN {
        1.0 + (AFTERGLOW - 1.0) * span(t, LINE, OPEN)
    } else {
        AFTERGLOW * (1.0 - ease_out(span(t, OPEN, 1.0)))
    }
}

pub fn screen_brightness(t: f32) -> f32 {
    1.0 + OVERSHOOT * screen_glow(t)
}

/// `full`, the picture's size across or down, opened `open` of the way from the line to all of
/// it, with what is left either side in whole art pixels so the picture stays on the grid.
fn on_grid(full: f32, open: f32) -> f32 {
    let size = LINE_PX + (full - LINE_PX).max(0.0) * open;
    let side = ((full - size) / 2.0 / LINE_PX).round() * LINE_PX;
    (full - 2.0 * side).clamp(LINE_PX.min(full), full)
}

/// Where a picture that fills `r` when the screen is on stands at power `t`, closing to a line
/// across the middle of `r`, then to a dot, as the screen goes off.
pub fn screen_rect_in(t: f32, r: [f32; 4]) -> (f32, f32, f32, f32) {
    let w = on_grid(r[2], ease_out(span(t, DOT, LINE)));
    let h = on_grid(r[3], ease_out(span(t, LINE, OPEN)));
    (r[0] + (r[2] - w) / 2.0, r[1] + (r[3] - h) / 2.0, w, h)
}
