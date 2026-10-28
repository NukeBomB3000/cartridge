//! What a notice looks like: a small panel across the top of the game in the theme's colours,
//! the game's or achievement's badge on its left and two lines of Pixel Code beside it, drawn at
//! art resolution and shown at two screen pixels to an art pixel.

use std::collections::VecDeque;
use std::time::Duration;

use cartridge_cheevos::badge::BADGE;
use cartridge_cheevos::Notice;
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::kit::pixel::{blank, fill, text_width, LINE};
use cartridge_ui::kit::text::text_face;
use cartridge_ui::UndoFace;

use crate::screens::lone::{fitted, over};

/// The panel's size in art pixels.
pub const FACE_W: u32 = 200;
pub const FACE_H: u32 = 46;
pub const SHOW: Duration = Duration::from_millis(4000);
pub const FADE: Duration = Duration::from_millis(250);

/// How far the badge is drawn, in art pixels.
const BADGE_PX: u32 = 30;
const PAD: u32 = 6;
const MAX_NOTICES: usize = 8;

/// A notice waiting to be shown: what it says and its badge, 64 by 64 pixels RGBA, if it had one.
pub struct Face {
    pub notice: Notice,
    pub badge: Option<Vec<u8>>,
    pub unlock: bool,
}

#[derive(Default)]
pub struct Queue(VecDeque<Face>);

impl Queue {
    /// Queues a notice; when too many wait, the oldest of the minor ones is dropped, never an
    /// unlock.
    pub fn push(&mut self, face: Face) {
        if self.0.len() >= MAX_NOTICES {
            match self.0.iter().position(|f| !f.unlock) {
                Some(i) => {
                    self.0.remove(i);
                }
                None if !face.unlock => return,
                None => {}
            }
        }
        self.0.push_back(face);
    }

    pub fn pop(&mut self) -> Option<Face> {
        self.0.pop_front()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// How opaque a notice is `shown_for` into its time: fading in, whole, fading out; none once
/// it is over.
pub fn alpha(shown_for: Duration) -> Option<f32> {
    if shown_for >= SHOW {
        return None;
    }
    let fade = FADE.as_secs_f32();
    let into = shown_for.as_secs_f32();
    let left = (SHOW - shown_for).as_secs_f32();
    Some((into / fade).min(left / fade).min(1.0))
}

/// The two things a notice says: a heading and a line under it.
pub fn lines(notice: &Notice) -> (String, String) {
    match notice {
        Notice::LoggedIn { user } => ("RetroAchievements".into(), format!("Signed in as {user}")),
        Notice::LoginFailed {
            needs_password: true,
            ..
        } => (
            "RetroAchievements login failed".into(),
            "Sign in again in Settings > Achievements".into(),
        ),
        Notice::LoginFailed { reason, .. } => {
            let hint = if reason.to_ascii_lowercase().contains("certificate") {
                "Check the clock".to_string()
            } else {
                reason.clone()
            };
            ("RetroAchievements login failed".into(), hint)
        }
        Notice::GameLoaded {
            title,
            unlocked,
            total,
            ..
        } => (
            title.clone(),
            if *total == 0 {
                "No achievements yet".into()
            } else {
                format!("{unlocked} of {total} achievements")
            },
        ),
        Notice::Unrecognised => (
            "Not recognised".into(),
            "RetroAchievements does not know this ROM".into(),
        ),
        Notice::Unlocked {
            title,
            description,
            points,
            ..
        } => (format!("{title} - {points} pts"), description.clone()),
        Notice::Mastered { title, .. } => ("Completed".into(), title.clone()),
        Notice::Offline => (
            "RetroAchievements offline".into(),
            "Unlocks are sent once back online".into(),
        ),
        Notice::Online => (
            "RetroAchievements back online".into(),
            "Pending unlocks sent".into(),
        ),
        Notice::ServerError { message } => ("RetroAchievements error".into(), message.clone()),
    }
}

/// `text` broken at spaces into at most two lines of `room` art pixels, the second cut short with
/// an ellipsis if it needs to be.
fn wrapped(text: &str, room: u32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let joined = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if current.is_empty() || text_width(&joined) <= room {
            current = joined;
        } else {
            lines.push(std::mem::replace(&mut current, word.to_string()));
        }
    }
    lines.push(current);
    if lines.len() > 2 {
        let rest = lines.split_off(1).join(" ");
        lines.push(rest);
    }
    lines.iter().map(|l| fitted(l, room)).collect()
}

/// The notice as a face in `pal`.
pub fn compose(face: &Face, pal: &Palette) -> UndoFace {
    let mut out = blank(FACE_W, FACE_H);
    let (w, h) = (FACE_W as i32, FACE_H as i32);
    fill(&mut out, [0, 0, w, h], pal.frame);
    fill(&mut out, [1, 1, w - 2, h - 2], pal.dark);
    let mut left = PAD;
    if let Some(badge) = face.badge.as_ref().filter(|b| b.len() == BADGE * BADGE * 4) {
        let top = (FACE_H - BADGE_PX) / 2;
        for y in 0..BADGE_PX {
            for x in 0..BADGE_PX {
                let sx = (x as usize * BADGE / BADGE_PX as usize).min(BADGE - 1);
                let sy = (y as usize * BADGE / BADGE_PX as usize).min(BADGE - 1);
                let px = &badge[(sy * BADGE + sx) * 4..][..4];
                if px[3] < 0x80 {
                    continue;
                }
                let at = (((top + y) * FACE_W + PAD + x) * 4) as usize;
                out.rgba[at..at + 4].copy_from_slice(&[px[0], px[1], px[2], 0xff]);
            }
        }
        left = PAD + BADGE_PX + PAD;
    }
    let room = FACE_W.saturating_sub(left + PAD);
    let (head, detail) = lines(&face.notice);
    let head = text_face(&fitted(&head, room), pal.frame_hi);
    over(&mut out, &head, left, 6);
    for (i, line) in wrapped(&detail, room).iter().enumerate() {
        let line = text_face(line, pal.cream);
        over(&mut out, &line, left, 6 + LINE + 3 + i as u32 * LINE);
    }
    out
}
