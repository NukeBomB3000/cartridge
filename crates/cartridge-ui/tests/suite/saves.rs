//! The switcher's SAVE STATES panel, kit::saves: a dot for every save, the one shown lit, as wide
//! as its title, its age or its dots need, and held to Chrome's cartridge-saves pixel for pixel.

use cartridge_store::RING_MAX;
use cartridge_ui::kit::saves::{dots_width, saves_face, saves_width, DOTS_TOP, SAVES_H, TITLE};
use cartridge_ui::UndoFace;

use crate::kit::{matches, RAMP};

/// The columns of the middle row of `face`'s dots that are `colour`, as runs `[from, to)`: the
/// well's inside starts 18 down.
fn runs(face: &UndoFace, colour: [u8; 3]) -> Vec<[u32; 2]> {
    let y = 18 + DOTS_TOP + 2;
    let at = |x: u32| {
        let i = ((y * face.w + x) * 4) as usize;
        face.rgba[i + 3] != 0 && face.rgba[i..i + 3] == colour
    };
    let mut out: Vec<[u32; 2]> = Vec::new();
    for x in 0..face.w {
        if !at(x) {
            continue;
        }
        match out.last_mut() {
            Some(run) if run[1] == x => run[1] = x + 1,
            _ => out.push([x, x + 1]),
        }
    }
    out
}

#[test]
fn there_is_a_dot_for_every_save_and_only_the_one_shown_is_lit() {
    for (count, at) in [(1, 0), (4, 1), (RING_MAX, RING_MAX - 1)] {
        let face = saves_face(saves_width("Just now", count), "Just now", count, at, &RAMP);
        let lit = runs(&face, RAMP.cream);
        let off = runs(&face, RAMP.plate);
        assert_eq!(lit.len() + off.len(), count, "{count} saves");
        // the well's inside starts 6 in; a dot is 5 wide and the next starts 8 on
        let dot = |i: usize| {
            let x = 6 + i as u32 * 8;
            [x, x + 5]
        };
        assert_eq!(lit, [dot(at)], "{count} saves, {at} shown");
    }
}

#[test]
fn the_panel_is_as_wide_as_its_title_its_age_or_its_dots_need() {
    // SAVE STATES in its tab, then the slant: 1 + (1 + 4 + 66 + 4) + 13 + 1
    assert_eq!(TITLE, "SAVE STATES");
    assert_eq!(saves_width("Just now", 1), 90);
    assert_eq!(saves_width("10 min ago", RING_MAX), 90);
    assert_eq!(dots_width(RING_MAX), 77);
    assert_eq!(saves_width("2026-08-08 20:00", 3), 6 + 96 + 6);
    // and the face is as wide as it is given, whatever it says
    for age in ["Just now", "59 min ago", "11 hr ago", "2026-12-31 23:59"] {
        let w = saves_width(age, RING_MAX);
        let face = saves_face(w, age, RING_MAX, 0, &RAMP);
        assert_eq!((face.w, face.h), (w, SAVES_H));
    }
}

#[test]
fn the_panel_is_chromes() {
    let face = |w, age, count, at| saves_face(w, age, count, at, &RAMP);
    matches(&face(90, "Just now", 1, 0), "saves-just-now");
    matches(&face(90, "4 min ago", 4, 1), "saves-four");
    matches(&face(90, "3 hr ago", 10, 9), "saves-ten");
    matches(&face(108, "2026-08-08 20:00", 3, 2), "saves-date");
    matches(&face(108, "Just now", 2, 0), "saves-held");
}
