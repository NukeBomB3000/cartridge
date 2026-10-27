//! The keyboard in the app: a handful of quads standing on the backdrop with no plate between,
//! for a passphrase and for an account alike, and moving with the slide that brings it in.

use crate::common;

use cartridge::app::{App, Phase};
use cartridge::label_screen::{AccountTyping, LabelsScreen, Step};
use cartridge::nav::Dir;
use cartridge::wifi_screen::{Typing, WifiScreen};
use cartridge_input::{Action, Btn};
use cartridge_store::{write_app_state, AppState};
use cartridge_ui::kit::layout::screen_px;
use cartridge_ui::{
    key_rect, keyboard_place, out_h, out_w, Draw, Keyboard, KeyboardFaces, TexId, FIELD_H, KEYS_H,
    KEYS_W, KEY_H, KEY_W,
};
use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use tempfile::TempDir;

fn app() -> (TempDir, App) {
    let d = tmp_root_with_carts(&["Emerald"]);
    write_app_state(
        d.path(),
        &AppState {
            clock_set: true,
            ..AppState::default()
        },
    )
    .unwrap();
    let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
    a.tick_ms(16);
    (d, a)
}

const WINDOW: TexId = TexId::from_raw(801);
const FIELD: TexId = TexId::from_raw(802);
const KEYS: [TexId; 3] = [
    TexId::from_raw(803),
    TexId::from_raw(804),
    TexId::from_raw(805),
];
const LIT: TexId = TexId::from_raw(806);

fn faces() -> KeyboardFaces {
    let [_, _, mw, mh] = keyboard_place().modal;
    KeyboardFaces {
        window: (WINDOW, mw, mh),
        field: (FIELD, KEYS_W, FIELD_H),
        keys: KEYS.map(|tex| (tex, KEYS_W, KEYS_H)),
        lit: (LIT, KEY_W, KEY_H),
    }
}

fn passphrase(keys: Keyboard) -> Phase {
    Phase::Network(WifiScreen {
        typing: Some(Typing {
            ssid: b"Home".to_vec(),
            replace: None,
            keys,
            refused: false,
        }),
        ..WifiScreen::default()
    })
}

fn account(keys: Keyboard) -> Phase {
    Phase::Labels(LabelsScreen {
        typing: Some(AccountTyping {
            step: Step::User,
            user: String::new(),
            keys,
        }),
        ..LabelsScreen::default()
    })
}

/// The frame `a` draws.
fn frame(a: &App) -> Vec<Draw> {
    let mut out = Vec::new();
    a.draw(&mut out);
    out
}

/// Where each of the keyboard's quads is drawn, in the order drawn: its texture, x and y.
fn keyboard_quads(out: &[Draw]) -> Vec<(TexId, f32, f32)> {
    let ours = [WINDOW, FIELD, KEYS[0], KEYS[1], KEYS[2], LIT];
    out.iter()
        .filter_map(|d| match *d {
            Draw::Tex { x, y, tex, .. } if ours.contains(&tex) => Some((tex, x, y)),
            _ => None,
        })
        .collect()
}

/// A see-through rect across the whole screen, as the plate under the other menus is.
fn plate(out: &[Draw]) -> bool {
    out.iter().any(|d| {
        matches!(*d, Draw::Rect { x, y, w, h, colour }
            if x == 0.0 && y == 0.0 && w == out_w() as f32 && h == out_h() as f32
                && colour[3] < 1.0)
    })
}

#[test]
fn either_keyboard_is_four_quads_on_the_backdrop_with_no_plate_under_it() {
    let p = keyboard_place();
    for phase in [passphrase, account] {
        let (_d, mut a) = app();
        a.set_keyboard_faces(faces());
        let keys = Keyboard::default();
        a.go(phase(keys.clone()), Dir::Deeper);
        a.finish_slide();
        let out = frame(&a);
        let [lx, ly, ..] = key_rect(keys.row, keys.col);
        assert_eq!(
            keyboard_quads(&out),
            [
                (WINDOW, screen_px(p.modal[0]), screen_px(p.modal[1])),
                (FIELD, screen_px(p.field[0]), screen_px(p.field[1])),
                (KEYS[0], screen_px(p.keys[0]), screen_px(p.keys[1])),
                (LIT, lx, ly),
            ],
            "{:?}",
            a.phase()
        );
        let window = out
            .iter()
            .position(|d| matches!(*d, Draw::Tex { tex, .. } if tex == WINDOW))
            .unwrap();
        assert!(
            matches!(out[window + 1..window + 4], [Draw::Tex { tex: f, .. }, Draw::Tex { tex: k, .. }, Draw::Tex { tex: l, .. }] if (f, k, l) == (FIELD, KEYS[0], LIT)),
            "something is drawn between the keyboard's quads"
        );
        assert!(!plate(&out), "a plate darkens the backdrop under the modal");
    }
}

#[test]
fn the_cursor_and_the_layer_pick_the_lit_key_and_the_keys_drawn() {
    let (_d, mut a) = app();
    a.set_keyboard_faces(faces());
    a.go(passphrase(Keyboard::default()), Dir::Deeper);
    a.finish_slide();
    for (press, layer) in [
        (Btn::Right, 0),
        (Btn::Down, 0),
        (Btn::L1, 2),
        (Btn::R1, 0),
        (Btn::X, 1),
    ] {
        a.apply(Action::GbaDown(press));
        let t = a.typing().expect("the keyboard went away");
        let [lx, ly, ..] = key_rect(t.keys.row, t.keys.col);
        let quads = keyboard_quads(&frame(&a));
        assert_eq!(
            quads[2].0, KEYS[layer],
            "{press:?} shows the wrong layer's keys"
        );
        assert_eq!(
            quads[3],
            (LIT, lx, ly),
            "{press:?}: the lit key is not on the cursor"
        );
    }
}

#[test]
fn the_keyboard_slides_in_whole() {
    let (_d, mut a) = app();
    a.set_keyboard_faces(faces());
    a.go(passphrase(Keyboard::default()), Dir::Deeper);
    let now = a.now();
    a.tick_ms(now + 60);
    let quads = keyboard_quads(&frame(&a));
    assert_eq!(quads.len(), 4);
    let p = keyboard_place();
    let dx = quads[0].1 - screen_px(p.modal[0]);
    assert!(dx != 0.0, "the keyboard did not slide");
    assert_eq!(
        quads[1].1 - screen_px(p.field[0]),
        dx,
        "the field slid apart"
    );
    assert_eq!(quads[2].1 - screen_px(p.keys[0]), dx, "the keys slid apart");
    assert_eq!(dx % 2.0, 0.0, "the keyboard slid off the art grid");
}
