/// `u`, from 0 to 1, eased in and out: slow off the start, slow onto the end.
pub fn ease(u: f32) -> f32 {
    u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
}
