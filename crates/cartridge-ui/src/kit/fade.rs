//! Fades in steps, the way the 16-bit consoles faded: a picture comes and goes in a few even
//! steps of brightness instead of smoothly, so nothing is ever half there for long.

/// How many steps a fade takes from nothing to all of it.
pub const STEPS: f32 = 4.0;

/// `amount`, from 0 to 1, moved to the nearest of the fade's steps: 0, 1/4, 1/2, 3/4 or 1.
pub fn stepped(amount: f32) -> f32 {
    (amount.clamp(0.0, 1.0) * STEPS).round() / STEPS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fade_takes_four_even_steps_and_keeps_its_ends() {
        assert_eq!(stepped(0.0), 0.0);
        assert_eq!(stepped(1.0), 1.0);
        assert_eq!(stepped(0.1), 0.0);
        assert_eq!(stepped(0.3), 0.25);
        assert_eq!(stepped(0.5), 0.5);
        assert_eq!(stepped(0.7), 0.75);
        assert_eq!(stepped(0.9), 1.0);
        assert_eq!(stepped(-1.0), 0.0);
        assert_eq!(stepped(2.0), 1.0);
    }
}
