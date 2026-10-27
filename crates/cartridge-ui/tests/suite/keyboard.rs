//! The on-screen keyboard: what its keys type and how the cursor walks them, and its faces,
//! held to screenshots of assets/ui/flow.html's keyboard pixel for pixel (keyboard/ref/, made
//! by keyboard/make-refs.sh). Set KEYBOARD_OUT to a folder to have each composed modal written
//! there as a PNG beside its reference.

use cartridge_store::colour_index;
use cartridge_ui::kit::layout::{modal_rect, screen_px};
use cartridge_ui::kit::palette::palette;
use cartridge_ui::kit::panel::panel_face;
use cartridge_ui::{
    field_face, key_box, key_label, key_on, key_rect, keyboard_place, keyboard_title, keys_face,
    lit_key_face, out_h, out_w, row_len, window_face, Draw, Key, Keyboard, KeyboardFaces,
    KeyboardLook, KeyboardScreen, KeyboardStale, Layer, TexId, UndoFace, BOTTOM, FIELD_H, KEYS_H,
    KEYS_W, KEY_COLS, KEY_GAP, KEY_H, KEY_ROWS, KEY_W, MAX_TEXT,
};

const LAYERS: [Layer; 3] = [Layer::Lower, Layer::Upper, Layer::Symbols];

fn amber() -> u8 {
    colour_index("amber").expect("amber is a backdrop colour")
}

#[test]
fn every_printable_ascii_character_is_on_some_layer() {
    let mut typed: Vec<char> = Vec::new();
    for layer in LAYERS {
        for row in 0..KEY_ROWS {
            for col in 0..row_len(row) {
                match key_on(layer, row, col) {
                    Key::Char(c) => typed.push(c),
                    Key::Space => typed.push(' '),
                    _ => {}
                }
            }
        }
    }
    for c in ' '..='~' {
        assert!(typed.contains(&c), "{c:?} cannot be typed");
    }
    assert!(
        typed.iter().all(|c| (' '..='~').contains(c)),
        "a key types something wpa_supplicant would refuse"
    );
}

#[test]
fn the_rows_of_characters_are_full_and_the_last_row_changes_what_they_type() {
    for row in 0..KEY_ROWS - 1 {
        assert_eq!(row_len(row), KEY_COLS);
    }
    assert_eq!(row_len(KEY_ROWS - 1), BOTTOM.len());
    for layer in LAYERS {
        for (col, key) in BOTTOM.iter().enumerate() {
            assert_eq!(key_on(layer, KEY_ROWS - 1, col), *key);
        }
    }
}

#[test]
fn keys_are_23_by_19_three_apart_and_the_bottom_row_spans_2_1_3_2_2_columns() {
    assert_eq!((KEY_W, KEY_H, KEY_GAP), (23, 19, 3));
    assert_eq!((KEYS_W, KEYS_H), (257, 107));
    for row in 0..KEY_ROWS - 1 {
        for col in 0..KEY_COLS {
            let (x, y) = (col as u32 * 26, row as u32 * 22);
            assert_eq!(key_box(row, col), [x, y, 23, 19], "key {row},{col}");
        }
    }
    let bottom: Vec<[u32; 4]> = (0..BOTTOM.len()).map(|c| key_box(4, c)).collect();
    assert_eq!(
        bottom,
        [
            [0, 88, 49, 19],
            [52, 88, 23, 19],
            [78, 88, 75, 19],
            [156, 88, 49, 19],
            [208, 88, 49, 19],
        ],
        "SHIFT, #+=, SPACE, DELETE and DONE"
    );
}

#[test]
fn every_key_is_inside_the_panels_well_above_the_hint_line_and_none_overlap() {
    let p = keyboard_place();
    let [wx, wy, ww, wh] = p.well;
    let mut rects = Vec::new();
    for row in 0..KEY_ROWS {
        for col in 0..row_len(row) {
            let [x, y, w, h] = key_rect(row, col);
            let inside = x >= screen_px(wx)
                && x + w <= screen_px(wx + ww)
                && y >= screen_px(wy)
                && y + h <= screen_px(wy + wh);
            assert!(inside, "key {row},{col} is outside the well");
            assert!(
                y + h <= screen_px(p.hints[1]),
                "key {row},{col} meets the hints"
            );
            assert!(
                x % 2.0 == 0.0 && y % 2.0 == 0.0,
                "key {row},{col} is off the art grid"
            );
            rects.push([x, y, w, h]);
        }
    }
    for (i, a) in rects.iter().enumerate() {
        for b in &rects[i + 1..] {
            let apart = a[0] + a[2] <= b[0]
                || b[0] + b[2] <= a[0]
                || a[1] + a[3] <= b[1]
                || b[1] + b[3] <= a[1];
            assert!(apart, "{a:?} overlaps {b:?}");
        }
    }
}

#[test]
fn the_keyboard_sits_in_the_modal_where_flow_html_centres_it() {
    let p = keyboard_place();
    assert_eq!(p.modal, modal_rect());
    let [mx, my, mw, mh] = p.modal;
    assert_eq!(
        p.panel,
        [4, 2, mw - 8, mh - 6],
        "the one panel fills the modal"
    );
    let [px, py, pw, ph] = p.panel;
    let well = panel_face(pw, ph, "PASSPHRASE FOR HOME", true, palette(0)).well;
    assert_eq!(
        p.well,
        [mx + px + well[0], my + py + well[1], well[2], well[3]],
        "not the panel's own well"
    );
    // down its well, the field, 6, the keys, 6 and a line of hints, offsets rounded down
    let [wx, wy, ww, wh] = p.well;
    let (x, y) = (wx + (ww - KEYS_W) / 2, wy + (wh - 144) / 2);
    assert_eq!(p.field, [x, y, KEYS_W, FIELD_H]);
    assert_eq!(p.keys, [x, y + 21, KEYS_W, KEYS_H]);
    assert_eq!(
        p.hints,
        [x + 15, y + 134],
        "B DELETE  X SHIFT  Y SPACE  START DONE"
    );
    if (out_w(), out_h()) == (640, 480) {
        assert_eq!(
            (p.field[0], p.field[1]),
            (31, 50),
            "flow.html's field at 640x480"
        );
    }
}

#[test]
fn the_bottom_row_spans_the_rows_above_it() {
    let first = key_rect(0, 0);
    let last = key_rect(0, KEY_COLS - 1);
    let left = key_rect(KEY_ROWS - 1, 0);
    let right = key_rect(KEY_ROWS - 1, BOTTOM.len() - 1);
    assert_eq!(left[0], first[0]);
    assert_eq!(right[0] + right[2], last[0] + last[2]);
}

#[test]
fn the_cursor_starts_on_q_and_wraps_round_each_row_and_column() {
    let mut k = Keyboard::default();
    assert_eq!(k.key(), Key::Char('q'));
    k.left();
    assert_eq!(k.key(), Key::Char('p'));
    k.right();
    assert_eq!(k.key(), Key::Char('q'));
    k.up();
    k.up();
    assert_eq!(
        k.row,
        KEY_ROWS - 1,
        "up from the top did not wrap to the bottom"
    );
    k.down();
    assert_eq!(k.row, 0);
}

#[test]
fn moving_between_rows_lands_on_the_key_nearest_below() {
    let mut k = Keyboard::default();
    k.down();
    k.down();
    for _ in 0..4 {
        k.right();
    }
    assert_eq!(k.key(), Key::Char('b'));
    k.down();
    assert_eq!(k.key(), Key::Space, "b sits over the space bar");
    k.right();
    assert_eq!(k.key(), Key::Delete);
    // Delete spans m and _, as near as each other: the left one, as flow.html walks it
    k.up();
    assert_eq!(
        k.key(),
        Key::Char('m'),
        "the cursor did not go back up over Delete"
    );
    k.right();
    k.down();
    assert_eq!(k.key(), Key::Delete, "_ is not over Delete");
}

#[test]
fn shift_and_symbols_change_what_the_rows_type_and_go_back() {
    let mut k = Keyboard::default();
    k.shift();
    assert_eq!(k.key(), Key::Char('Q'));
    k.shift();
    assert_eq!(k.key(), Key::Char('q'));
    k.symbols();
    assert_eq!(k.layer, Layer::Symbols);
    k.shift();
    assert_eq!(
        k.layer,
        Layer::Lower,
        "shift on symbols did not go back to letters"
    );
    k.symbols();
    k.symbols();
    assert_eq!(k.layer, Layer::Lower);
}

#[test]
fn pressing_types_changes_layer_or_finishes() {
    let mut k = Keyboard::default();
    assert!(!k.press());
    assert_eq!(k.text, "q");
    k.row = KEY_ROWS - 1;
    k.col = BOTTOM.iter().position(|b| *b == Key::Shift).unwrap();
    assert!(!k.press());
    assert_eq!(k.layer, Layer::Upper);
    k.col = BOTTOM.iter().position(|b| *b == Key::Space).unwrap();
    k.press();
    k.col = BOTTOM.iter().position(|b| *b == Key::Delete).unwrap();
    k.press();
    k.press();
    assert_eq!(k.text, "");
    k.col = BOTTOM.iter().position(|b| *b == Key::Done).unwrap();
    assert!(k.press(), "Join did not finish");
}

#[test]
fn the_field_takes_no_more_than_wpa_supplicant_will() {
    let mut k = Keyboard::default();
    for _ in 0..MAX_TEXT + 5 {
        k.press();
    }
    assert_eq!(k.text.len(), MAX_TEXT);
}

#[test]
fn keys_say_their_character_in_its_case_and_the_bottom_row_its_word() {
    assert_eq!(key_label(Layer::Lower, 1, 0), "q");
    assert_eq!(key_label(Layer::Upper, 1, 0), "Q");
    assert_eq!(key_label(Layer::Symbols, 2, 0), "\\");
    let words = |layer| (0..BOTTOM.len()).map(move |c| key_label(layer, 4, c));
    let lower: Vec<String> = words(Layer::Lower).collect();
    assert_eq!(lower, ["SHIFT", "#+=", "SPACE", "DELETE", "DONE"]);
    assert_eq!(
        key_label(Layer::Symbols, 4, 1),
        "abc",
        "no way back to letters"
    );
}

#[test]
fn the_title_is_the_prompt_in_capitals_cut_short_where_it_would_run_out() {
    assert_eq!(
        keyboard_title("Passphrase for Home", 280),
        "PASSPHRASE FOR HOME"
    );
    let long = format!("Passphrase for {}", "A".repeat(32));
    let cut = keyboard_title(&long, 280);
    assert!(cut.ends_with("..."), "{cut}");
    assert!(cut.starts_with("PASSPHRASE FOR AAA"), "{cut}");
    assert!(
        cartridge_ui::kit::panel::tab_width(&cut) + 1 + 13 < 280,
        "{cut} runs its tab past the panel"
    );
    assert_eq!(keyboard_title(&long, 400), long.to_uppercase());
}

#[test]
fn each_layers_keys_are_a_face_of_their_own() {
    let pal = palette(0);
    let faces = LAYERS.map(|layer| keys_face(layer, pal));
    for face in &faces {
        assert_eq!((face.w, face.h), (KEYS_W, KEYS_H));
    }
    assert_ne!(faces[0].rgba, faces[1].rgba, "shift shows the same keys");
    assert_ne!(faces[0].rgba, faces[2].rgba, "symbols show the same keys");
    assert_ne!(faces[1].rgba, faces[2].rgba);
    // the digits are the same small or capital, and so is the bottom row but its #+=
    let rows = |face: &UndoFace, from: u32, to: u32| {
        face.rgba[(from * face.w * 4) as usize..(to * face.w * 4) as usize].to_vec()
    };
    assert_eq!(
        rows(&faces[0], 0, 19),
        rows(&faces[1], 0, 19),
        "shift moved the digits"
    );
    assert_eq!(rows(&faces[0], 88, 107), rows(&faces[1], 88, 107));
    assert_ne!(rows(&faces[0], 88, 107), rows(&faces[2], 88, 107), "no abc");
}

#[test]
fn a_new_palette_tints_every_face_again() {
    let (night, amber) = (palette(0), palette(amber()));
    let pairs = [
        (
            window_face("Passphrase for Home", night),
            window_face("Passphrase for Home", amber),
        ),
        (
            keys_face(Layer::Lower, night),
            keys_face(Layer::Lower, amber),
        ),
        (
            field_face("hunter2", false, night),
            field_face("hunter2", false, amber),
        ),
        (
            lit_key_face(Layer::Lower, 1, 0, night),
            lit_key_face(Layer::Lower, 1, 0, amber),
        ),
    ];
    for (a, b) in &pairs {
        assert_eq!((a.w, a.h), (b.w, b.h));
        assert_ne!(a.rgba, b.rgba, "a face kept Night's colours");
    }
    // the keys in amber are drawn in amber's tab, outline and cream only
    let keys = &pairs[1].1;
    for px in keys.rgba.chunks(4).filter(|p| p[3] != 0) {
        let rgb = [px[0], px[1], px[2]];
        assert!(
            [amber.tab, amber.dark, amber.cream].contains(&rgb),
            "{rgb:?} is not one of amber's key colours"
        );
    }
}

#[test]
fn the_lit_key_is_a_face_its_keys_size_with_a_bright_face_and_dark_label() {
    let pal = palette(0);
    for (row, col) in [(0, 0), (4, 0), (4, 2)] {
        let face = lit_key_face(Layer::Lower, row, col, pal);
        let [_, _, w, h] = key_box(row, col);
        assert_eq!((face.w, face.h), (w, h));
        let at = |x: u32, y: u32| {
            let i = ((y * face.w + x) * 4) as usize;
            [
                face.rgba[i],
                face.rgba[i + 1],
                face.rgba[i + 2],
                face.rgba[i + 3],
            ]
        };
        assert_eq!(at(0, 0)[3], 0, "the corner is not left out");
        assert_eq!(at(0, 9)[..3], pal.dark, "no outline");
        assert_eq!(at(1, 1)[..3], pal.frame_hi, "not lit");
        let inked = (0..h).any(|y| (0..w).any(|x| at(x, y)[..3] == pal.dark && x > 1));
        assert!(inked, "no label on the lit key");
    }
}

#[test]
fn the_field_shows_the_end_of_what_is_typed_with_the_cursor_after_it() {
    let pal = palette(0);
    let at = |face: &UndoFace, x: u32, y: u32| {
        let i = ((y * face.w + x) * 4) as usize;
        [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2]]
    };
    let empty = field_face("", false, pal);
    assert_eq!((empty.w, empty.h), (KEYS_W, FIELD_H));
    assert_eq!(at(&empty, 0, 7), pal.edge);
    assert_eq!(at(&empty, 5, 4), pal.frame_hi, "no cursor at the start");
    let typed = field_face("abc", false, pal);
    assert_eq!(
        at(&typed, 5 + 18, 4),
        pal.frame_hi,
        "the cursor is not after abc"
    );
    let cream = |face: &UndoFace| (2..13).any(|y| (5..23).any(|x| at(face, x, y) == pal.cream));
    assert!(
        cream(&typed) && !cream(&empty),
        "abc is not in the field, in cream"
    );
    let long: String = ('a'..='z').cycle().take(MAX_TEXT).collect();
    let tail: String = long.chars().skip(MAX_TEXT - 40).collect();
    assert_eq!(
        field_face(&long, false, pal).rgba,
        field_face(&tail, false, pal).rgba
    );
    assert_eq!(at(&field_face(&tail, false, pal), 5 + 240, 4), pal.frame_hi);
    let refused = field_face("abc", true, pal);
    assert_eq!(
        at(&refused, 0, 7),
        pal.frame_hi,
        "the field does not say no"
    );
}

#[test]
fn typing_bakes_the_field_and_moving_bakes_the_lit_key_and_nothing_else() {
    let mut keys = Keyboard::default();
    let look =
        |keys: &Keyboard, prompt: &str, colour: u8| KeyboardLook::new(prompt, keys, false, colour);
    let baked = look(&keys, "Passphrase for Home", 0);
    assert_eq!(
        baked.stale(None),
        KeyboardStale::ALL,
        "nothing baked to start"
    );
    assert!(
        !baked.stale(Some(&baked)).any(),
        "the same look bakes again"
    );

    keys.press();
    let typed = look(&keys, "Passphrase for Home", 0);
    let nothing = KeyboardStale {
        window: false,
        keys: false,
        field: false,
        lit: false,
    };
    assert_eq!(
        typed.stale(Some(&baked)),
        KeyboardStale {
            field: true,
            ..nothing
        },
        "typing bakes more than the field"
    );
    keys.right();
    keys.shift();
    let moved = look(&keys, "Passphrase for Home", 0);
    assert_eq!(
        moved.stale(Some(&typed)),
        KeyboardStale {
            lit: true,
            ..nothing
        },
        "moving or shifting bakes more than the lit key"
    );
    let asked = look(&keys, "ScreenScraper password for me", 0);
    assert_eq!(
        asked.stale(Some(&moved)),
        KeyboardStale {
            window: true,
            ..nothing
        },
        "a new prompt bakes more than the window"
    );
    let refused = KeyboardLook::new("A passphrase has 8 to 63 characters", &keys, true, 0);
    let stale = refused.stale(Some(&moved));
    assert!(stale.window && stale.field && !stale.keys && !stale.lit);
    assert_eq!(
        look(&keys, "Passphrase for Home", amber()).stale(Some(&moved)),
        KeyboardStale::ALL,
        "a new palette leaves a face in the old one"
    );
}

#[test]
fn drawing_the_keyboard_is_four_quads_the_lit_key_over_the_cursors_key() {
    let p = keyboard_place();
    let tex = |n: usize| TexId::from_raw(n);
    let faces = KeyboardFaces {
        window: (tex(1), p.modal[2], p.modal[3]),
        field: (tex(2), KEYS_W, FIELD_H),
        keys: [
            (tex(3), KEYS_W, KEYS_H),
            (tex(4), KEYS_W, KEYS_H),
            (tex(5), KEYS_W, KEYS_H),
        ],
        lit: (tex(6), KEY_W, KEY_H),
    };
    for (layer, row, col) in [
        (Layer::Lower, 1, 0),
        (Layer::Upper, 3, 9),
        (Layer::Symbols, 4, 2),
    ] {
        let keys = Keyboard {
            layer,
            row,
            col,
            ..Keyboard::default()
        };
        let mut out = Vec::new();
        KeyboardScreen {
            keys: &keys,
            faces: Some(&faces),
        }
        .draw(&mut out);
        let quads: Vec<(TexId, [f32; 4])> = out
            .iter()
            .map(|d| match *d {
                Draw::Tex {
                    x,
                    y,
                    w,
                    h,
                    tex,
                    alpha,
                } => {
                    assert_eq!(alpha, 1.0);
                    (tex, [x, y, w, h])
                }
                other => panic!("the keyboard drew {other:?}"),
            })
            .collect();
        let at = |r: [u32; 4]| r.map(screen_px);
        let [kx, ky] = [p.keys[0], p.keys[1]];
        let [x, y, ..] = key_box(row, col);
        assert_eq!(
            quads,
            [
                (tex(1), at(p.modal)),
                (tex(2), at(p.field)),
                (tex(3 + layer.index()), at(p.keys)),
                (tex(6), at([kx + x, ky + y, KEY_W, KEY_H])),
            ],
            "{layer:?} with the cursor on {row},{col}"
        );
        let [lx, ly, ..] = key_rect(row, col);
        assert_eq!(
            [quads[3].1[0], quads[3].1[1]],
            [lx, ly],
            "the lit key is not over its key"
        );
    }
    let mut out = Vec::new();
    let keys = Keyboard::default();
    KeyboardScreen {
        keys: &keys,
        faces: None,
    }
    .draw(&mut out);
    assert!(out.is_empty(), "drew a keyboard with no faces");
}

/// A reference in keyboard/ref/ as RGB rows, whatever the PNG stores.
fn reference(name: &str) -> Option<(u32, u32, Vec<[u8; 3]>)> {
    let path = format!(
        "{}/tests/suite/keyboard/ref/{name}.png",
        env!("CARGO_MANIFEST_DIR")
    );
    let file = std::fs::File::open(&path).ok()?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().expect("a PNG");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("its pixels");
    let step = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => panic!("{name} is {other:?}"),
    };
    let px = buf[..info.buffer_size()]
        .chunks(step)
        .map(|p| [p[0], p[1], p[2]])
        .collect();
    Some((info.width, info.height, px))
}

/// `src` over `dst` at (`x`, `y`), as the compositor lays one quad on another.
fn over(dst: &mut UndoFace, src: &UndoFace, x: u32, y: u32) {
    for sy in 0..src.h {
        for sx in 0..src.w {
            let from = ((sy * src.w + sx) * 4) as usize;
            if src.rgba[from + 3] == 0 {
                continue;
            }
            let to = (((y + sy) * dst.w + x + sx) * 4) as usize;
            dst.rgba[to..to + 4].copy_from_slice(&src.rgba[from..from + 4]);
        }
    }
}

#[test]
fn the_keyboard_is_flow_html_s_pixel_for_pixel() {
    let canvas = format!("{}x{}", out_w(), out_h());
    // flow.html's row= is the key's row and, on the bottom row, its column of ten
    let shots = [
        ("640x480", "night", (0, 0), (0, 0)),
        ("640x480", "amber", (4, 3), (4, 2)),
        ("720x480", "night", (2, 5), (2, 5)),
        ("720x480", "amber", (3, 9), (3, 9)),
        ("720x720", "night", (1, 0), (1, 0)),
        ("720x720", "amber", (4, 8), (4, 4)),
    ];
    let p = keyboard_place();
    let [mx, my, mw, mh] = p.modal;
    let mut held = 0;
    for (size, colour, flow, (row, col)) in shots {
        if size != canvas {
            continue;
        }
        let name = format!("keyboard-{size}-{colour}-{}-{}", flow.0, flow.1);
        let (w, h, want) = reference(&name).unwrap_or_else(|| panic!("no {name}.png"));
        assert_eq!((w, h), (mw, mh), "{name} is not the modal's size");
        let pal = palette(colour_index(colour).expect("a backdrop colour"));
        let mut drawn = UndoFace {
            rgba: vec![0; (mw * mh * 4) as usize],
            w: mw,
            h: mh,
        };
        over(&mut drawn, &window_face("Passphrase for Home", pal), 0, 0);
        over(
            &mut drawn,
            &field_face("", false, pal),
            p.field[0] - mx,
            p.field[1] - my,
        );
        let [kx, ky] = [p.keys[0] - mx, p.keys[1] - my];
        over(&mut drawn, &keys_face(Layer::Lower, pal), kx, ky);
        let [x, y, ..] = key_box(row, col);
        over(
            &mut drawn,
            &lit_key_face(Layer::Lower, row, col, pal),
            kx + x,
            ky + y,
        );
        if let Ok(dir) = std::env::var("KEYBOARD_OUT") {
            let file = std::fs::File::create(format!("{dir}/{name}.png")).unwrap();
            let mut e = png::Encoder::new(std::io::BufWriter::new(file), mw, mh);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .unwrap()
                .write_image_data(&drawn.rgba)
                .unwrap();
        }
        let (mut clear, mut wrong) = (0, Vec::new());
        for (i, (got, want)) in drawn.rgba.chunks(4).zip(&want).enumerate() {
            if got[3] == 0 {
                // the modal's cut corners, where the backdrop shows
                clear += 1;
            } else if got[..3] != want[..] {
                wrong.push((i as u32 % mw, i as u32 / mw, &got[..3], *want));
            }
        }
        assert_eq!(
            clear,
            4 * 5,
            "{name}: clear but for the modal's four cut corners"
        );
        assert!(
            wrong.is_empty(),
            "{name}: {} pixels differ, first {:?}",
            wrong.len(),
            &wrong[..wrong.len().min(4)]
        );
        held += 1;
    }
    if ["640x480", "720x480", "720x720"].contains(&canvas.as_str()) {
        assert_eq!(held, 2, "no reference for {canvas}");
    }
}
