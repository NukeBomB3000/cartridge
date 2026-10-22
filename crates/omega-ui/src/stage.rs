use omega_gfx::{out_h, out_w};

/// The screens were drawn on a 720x480 mockup, and their scenes still are: on another canvas
/// the mockup sits centred on it, moved by these offsets, so on the RG CubeXX's square canvas
/// the shelf stands between the bars and not down by the footer. On 720x480 both are 0.
pub const MOCKUP_W: f32 = 720.0;
pub const MOCKUP_H: f32 = 480.0;

pub fn stage_x() -> f32 {
    (out_w() as f32 - MOCKUP_W) / 2.0
}

pub fn stage_y() -> f32 {
    (out_h() as f32 - MOCKUP_H) / 2.0
}
