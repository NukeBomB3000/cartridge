use crate::hud::Millis;
use crate::sprite::SPRITE_PX;

const REFUSAL_MS: Millis = 300;

const SHAKE_PX: f32 = 6.0;
const SHAKE_HZ: f32 = 14.0;

#[derive(Copy, Clone)]
pub struct Refusal {
    started: Millis,
}

impl Refusal {
    pub fn started(now: Millis) -> Self {
        Refusal { started: now }
    }

    pub fn active(&self, now: Millis) -> bool {
        now.saturating_sub(self.started) < REFUSAL_MS
    }

    /// How far across the shake has moved what it shakes `now`: a cosine dying away, moved to
    /// the nearest whole art pixel, so what shakes jumps an art pixel at a time and stays on
    /// the grid the rest of the UI is drawn on.
    pub fn offset(&self, now: Millis) -> f32 {
        let age = now.saturating_sub(self.started);
        if age >= REFUSAL_MS {
            return 0.0;
        }
        let decay = 1.0 - age as f32 / REFUSAL_MS as f32;
        let secs = age as f32 / 1000.0;
        let swing = (secs * SHAKE_HZ * std::f32::consts::TAU).cos() * SHAKE_PX * decay;
        let art = SPRITE_PX as f32;
        (swing / art).round() * art
    }
}
