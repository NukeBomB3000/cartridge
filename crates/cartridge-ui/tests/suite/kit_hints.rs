//! The keys on a line of their own (kit/hints.rs), held to screenshots of cartridge-hints in Chrome
//! (kit/ref/, made by kit/make-refs.sh), pixel for pixel, as kit.rs holds the menus' faces: a
//! gauge's chrome with the bar's hints in its well.

use cartridge_ui::kit::bar::Hint;
use cartridge_ui::kit::gauge::{gauge_face, well_x, GAUGE_H};
use cartridge_ui::kit::hints::{hint_line_face, hint_line_width, HINT_LINE_H};
use cartridge_ui::UndoFace;

use crate::kit::{matches, RAMP};

const DELETE: Hint = Hint {
    key: "Y",
    word: "DELETE",
};
const UNDO_SAVE: Hint = Hint {
    key: "X",
    word: "UNDO SAVE",
};
const UNDO_LOAD: Hint = Hint {
    key: "X",
    word: "UNDO LOAD",
};

fn line(label: &str, hints: &[Hint]) -> UndoFace {
    hint_line_face(hint_line_width(label, hints), label, hints, &RAMP)
}

#[test]
fn the_hint_lines_are_chromes() {
    matches(&line("SAVE", &[DELETE, UNDO_SAVE]), "hints-switcher");
    matches(&line("SAVE", &[DELETE, UNDO_LOAD]), "hints-switcher-load");
    matches(&line("SAVE", &[DELETE]), "hints-switcher-delete");
    let held = hint_line_width("SAVE", &[DELETE, UNDO_SAVE]);
    matches(
        &hint_line_face(held, "SAVE", &[DELETE], &RAMP),
        "hints-switcher-held",
    );
    matches(&line("KEYS", &[]), "hints-empty");
    let three = [
        Hint {
            key: "START",
            word: "FETCH",
        },
        Hint {
            key: "Y",
            word: "SIGN OUT",
        },
        Hint {
            key: "SELECT",
            word: "",
        },
    ];
    matches(&line("keys", &three), "hints-three");
}

#[test]
fn a_line_is_as_wide_as_its_title_and_its_keys_ink() {
    // the frame either side, the tab (its frame line, 4 either side of the title), the slant 14,
    // the well's shadow lines and 2 inside either side, the strip's pixel after it
    let chrome = 1 + (1 + 4 + 24 + 4) + 14 + (1 + 2 + 2 + 1) + 1 + 1;
    assert_eq!(hint_line_width("SAVE", &[]), chrome);
    assert_eq!(hint_line_width("SAVE", &[DELETE]), chrome + 47);
    assert_eq!(hint_line_width("SAVE", &[DELETE, UNDO_SAVE]), chrome + 120);
    assert_eq!(
        hint_line_width("SAVE", &[DELETE, UNDO_LOAD]),
        hint_line_width("SAVE", &[DELETE, UNDO_SAVE])
    );
    assert_eq!(HINT_LINE_H, GAUGE_H);
    let narrow = hint_line_face(10, "SAVE", &[DELETE], &RAMP);
    assert_eq!(
        (narrow.w, narrow.h),
        (hint_line_width("SAVE", &[DELETE]), HINT_LINE_H),
        "a line given less room than it needs"
    );
}

/// The line is a gauge with the keys in its well for the cells: every pixel up to the well's
/// inside is the gauge's own, the frame, the tab, its title and the slant.
#[test]
fn a_line_is_a_gauge_up_to_its_well() {
    let hints = line("SAVE", &[DELETE, UNDO_SAVE]);
    let gauge = gauge_face("SAVE", 0, 9, 9, None, &RAMP);
    let to = well_x("SAVE") + 1;
    for y in 0..GAUGE_H {
        for x in 0..to {
            let at = |f: &UndoFace| {
                let i = ((y * f.w + x) * 4) as usize;
                f.rgba[i..i + 4].to_vec()
            };
            assert_eq!(at(&hints), at(&gauge), "({x}, {y})");
        }
    }
}

/// The keys' capitals are on the title's rows, 4 to 10, and each cap fills the well's inside,
/// rows 3 to 11: nothing of theirs is drawn on the well's shadow line or past it.
#[test]
fn the_keys_stand_on_the_titles_rows_and_fill_the_well() {
    let face = line("SAVE", &[DELETE, UNDO_SAVE]);
    // from inside the well's left shadow line to its right one: the frame, the strip's pixel
    // and the shadow line are the face's last three columns
    let inside = well_x("SAVE") + 1..face.w - 3;
    let (mut caps, mut ink) = (Vec::new(), Vec::new());
    for y in 0..face.h {
        for x in 0..face.w {
            let i = ((y * face.w + x) * 4) as usize;
            let colour = [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2]];
            if colour == RAMP.dark {
                caps.push((x, y));
            } else if colour == RAMP.cream {
                ink.push((x, y));
            }
        }
    }
    assert!(caps
        .iter()
        .all(|(x, y)| (3..=11).contains(y) && inside.contains(x)));
    assert_eq!(
        (3..=11)
            .filter(|y| caps.iter().any(|(_, cy)| cy == y))
            .count(),
        9,
        "a cap misses a row of the well's inside"
    );
    assert!(ink.iter().all(|(_, y)| (4..=10).contains(y)), "{ink:?}");
    let well_ink = ink.iter().filter(|(x, _)| inside.contains(x));
    assert_eq!(
        well_ink.clone().map(|(x, _)| *x).min(),
        Some(well_x("SAVE") + 1 + 2 + 2),
        "a cap's letters start two dark pixels in"
    );
}
