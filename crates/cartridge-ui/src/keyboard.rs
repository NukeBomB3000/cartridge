//! An on-screen keyboard for typing a passphrase with the d-pad: four rows of characters
//! and a row of keys that change what they type, on three layers between them covering
//! every printable ASCII character.
//!
//! It is drawn as assets/ui/flow.html's keyboard: a modal holding one panel titled with what
//! it asks for, and in the panel's well the field, the keys and a line of hints, centred. Keys
//! are 23 by 19 art pixels, 3 apart, each a 9-slice with a dark outline, its label in Pixel
//! Code; the bottom row's keys span 2, 1, 3, 2 and 2 columns.
//!
//! Drawing it is four quads: the window (the modal, its panel and the hint line), the field,
//! the keys of the layer showing, and the key under the cursor lit over them. Each layer's keys
//! are one face, baked once for each palette, so typing bakes the field again and nothing else,
//! and moving the cursor bakes the one lit key ([`KeyboardLook::stale`] says which).

use crate::draw::{Draw, TexId};
use crate::kit::layout::{modal_rect, screen_px, PANEL_CHROME_H};
use crate::kit::modal::{modal_face, modal_layout};
use crate::kit::palette::Palette;
use crate::kit::panel::panel_face;
use crate::kit::pixel::{blank, fill, mirror, nine_slice, quad, text_width, ADVANCE, CAP, LINE};
use crate::kit::text::text_face;
use crate::plate::UndoFace;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Layer {
    Lower,
    Upper,
    Symbols,
}

impl Layer {
    pub const ALL: [Layer; 3] = [Layer::Lower, Layer::Upper, Layer::Symbols];

    /// Where the layer is in [`Layer::ALL`], and so which of `KeyboardFaces::keys` it draws.
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Key {
    Char(char),
    Shift,
    Symbols,
    Space,
    Delete,
    Done,
}

/// The keys that change what the others type, under the four rows of characters.
pub const BOTTOM: [Key; 5] = [Key::Shift, Key::Symbols, Key::Space, Key::Delete, Key::Done];
/// How many columns of the rows above each of them spans.
const BOTTOM_SPAN: [u32; 5] = [2, 1, 3, 2, 2];

const LOWER: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl-", "zxcvbnm_.@"];
const UPPER: [&str; 4] = ["1234567890", "QWERTYUIOP", "ASDFGHJKL-", "ZXCVBNM_.@"];
const SYMBOLS: [&str; 4] = ["!@#$%^&*()", "`~-_=+[]{}", "\\|;:'\",.<>", "/?!@#$%&*+"];

pub const KEY_COLS: usize = 10;
pub const KEY_ROWS: usize = 5;
/// The longest passphrase wpa_supplicant takes.
pub const MAX_TEXT: usize = 63;

/// A key, in art pixels: odd sizes, so a label's ink (an odd width, 7 tall) has as much room
/// on either side, and above as below.
pub const KEY_W: u32 = 23;
pub const KEY_H: u32 = 19;
/// The room between two keys, across and down.
pub const KEY_GAP: u32 = 3;
/// All the keys together, 10 across and 5 down: 257 by 107 art pixels.
pub const KEYS_W: u32 = KEY_COLS as u32 * (KEY_W + KEY_GAP) - KEY_GAP;
pub const KEYS_H: u32 = KEY_ROWS as u32 * (KEY_H + KEY_GAP) - KEY_GAP;
/// The field over the keys, as wide as they are: a line inside a frame line and a pixel of
/// room above it.
pub const FIELD_H: u32 = 15;
/// The room between the field, the keys and the hint line.
const STACK_GAP: u32 = 6;
/// The field, the keys and the hint line one over the other: what the panel's well centres.
const STACK_H: u32 = FIELD_H + STACK_GAP + KEYS_H + STACK_GAP + LINE;
/// Where a panel's well starts inside the panel (kit/panel.rs): its stripe, the frame line,
/// the well's shadow line and its padding, across; the header and the same lines, down.
const WELL_X: u32 = 6;
const WELL_Y: u32 = 18;

/// Inside the field: its frame line and four pixels of room before the text, and the frame line
/// and a pixel of room over the text's line.
const FIELD_TEXT: [u32; 2] = [1 + 4, 1 + 1];
/// How much of the end of what is typed the field shows, as flow.html: 40 letters and the cursor
/// after them fill its 247 pixels inside.
const FIELD_CHARS: usize = 40;

/// The buttons that do a key's work wherever the cursor is, as the hint line says them.
const HINTS: [(&str, &str); 4] = [
    ("B", "DELETE"),
    ("X", "SHIFT"),
    ("Y", "SPACE"),
    ("START", "DONE"),
];
/// The room between one hint and the next, in letters.
const HINT_GAP: usize = 2;

/// A key: its corner, mirrored into a 9-slice, a dark outline with the corner pixel left out
/// around the key's face (assets/ui/flow/art.js). The lit key has a bright face.
#[rustfmt::skip]
const KEY: [&str; 2] = [
    ".k",
    "kt",
];
#[rustfmt::skip]
const KEY_LIT: [&str; 2] = [
    ".k",
    "kH",
];

/// The room a panel's title leaves in its panel: the panel's stripe and the tab's frame line,
/// the tab's padding either side of the title, its slanted end and the stripe on the far side.
const TITLE_ROOM: u32 = 1 + 1 + 2 * 4 + 13 + 1;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Keyboard {
    pub text: String,
    pub layer: Layer,
    pub row: usize,
    pub col: usize,
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard {
            text: String::new(),
            layer: Layer::Lower,
            row: 1,
            col: 0,
        }
    }
}

impl Keyboard {
    pub fn key_at(&self, row: usize, col: usize) -> Key {
        key_on(self.layer, row, col)
    }

    pub fn key(&self) -> Key {
        self.key_at(self.row, self.col)
    }

    pub fn left(&mut self) {
        let n = row_len(self.row);
        self.col = (self.col + n - 1) % n;
    }

    pub fn right(&mut self) {
        self.col = (self.col + 1) % row_len(self.row);
    }

    pub fn up(&mut self) {
        self.go_to((self.row + KEY_ROWS - 1) % KEY_ROWS);
    }

    pub fn down(&mut self) {
        self.go_to((self.row + 1) % KEY_ROWS);
    }

    /// Moves to another row, onto the key nearest straight above or below; of two as near,
    /// the left one.
    fn go_to(&mut self, row: usize) {
        let [x, _, w, _] = key_box(self.row, self.col);
        let centre = 2 * x + w;
        self.col = (0..row_len(row))
            .min_by_key(|&c| {
                let [x, _, w, _] = key_box(row, c);
                (2 * x + w).abs_diff(centre)
            })
            .unwrap_or(0);
        self.row = row;
    }

    /// Presses the key under the cursor. Returns true for the key that finishes typing.
    pub fn press(&mut self) -> bool {
        match self.key() {
            Key::Char(c) => self.type_char(c),
            Key::Shift => self.shift(),
            Key::Symbols => self.symbols(),
            Key::Space => self.type_char(' '),
            Key::Delete => self.delete(),
            Key::Done => return true,
        }
        false
    }

    pub fn type_char(&mut self, c: char) {
        if self.text.len() < MAX_TEXT {
            self.text.push(c);
        }
    }

    pub fn delete(&mut self) {
        self.text.pop();
    }

    pub fn shift(&mut self) {
        self.layer = match self.layer {
            Layer::Lower => Layer::Upper,
            Layer::Upper | Layer::Symbols => Layer::Lower,
        };
    }

    pub fn symbols(&mut self) {
        self.layer = match self.layer {
            Layer::Symbols => Layer::Lower,
            Layer::Lower | Layer::Upper => Layer::Symbols,
        };
    }
}

pub fn row_len(row: usize) -> usize {
    if row + 1 == KEY_ROWS {
        BOTTOM.len()
    } else {
        KEY_COLS
    }
}

pub fn key_on(layer: Layer, row: usize, col: usize) -> Key {
    if row + 1 == KEY_ROWS {
        return BOTTOM[col.min(BOTTOM.len() - 1)];
    }
    let rows = match layer {
        Layer::Lower => &LOWER,
        Layer::Upper => &UPPER,
        Layer::Symbols => &SYMBOLS,
    };
    Key::Char(rows[row].chars().nth(col).unwrap_or(' '))
}

/// Where a key is among the keys, `[x, y, w, h]` in art pixels from their top left: a
/// character's 23 by 19, a bottom key as wide as the columns it spans and the gaps between.
pub fn key_box(row: usize, col: usize) -> [u32; 4] {
    let pitch = KEY_W + KEY_GAP;
    let y = row as u32 * (KEY_H + KEY_GAP);
    if row + 1 < KEY_ROWS {
        return [col as u32 * pitch, y, KEY_W, KEY_H];
    }
    let col = col.min(BOTTOM.len() - 1);
    let first: u32 = BOTTOM_SPAN[..col].iter().sum();
    [first * pitch, y, BOTTOM_SPAN[col] * pitch - KEY_GAP, KEY_H]
}

/// Where a key is on the screen: x, y, w, h in screen pixels.
pub fn key_rect(row: usize, col: usize) -> [f32; 4] {
    let [kx, ky, ..] = keyboard_place().keys;
    let [x, y, w, h] = key_box(row, col);
    [
        screen_px(kx + x),
        screen_px(ky + y),
        screen_px(w),
        screen_px(h),
    ]
}

/// What a key in the bottom row says, by its place in [`BOTTOM`] and the layer showing.
pub fn bottom_label(key: Key, layer: Layer) -> &'static str {
    match key {
        Key::Shift => "SHIFT",
        Key::Symbols if layer == Layer::Symbols => "abc",
        Key::Symbols => "#+=",
        Key::Space => "SPACE",
        Key::Delete => "DELETE",
        Key::Done => "DONE",
        Key::Char(_) => "",
    }
}

/// What the key at `row`, `col` says on `layer`: its character in its own case, or its word.
pub fn key_label(layer: Layer, row: usize, col: usize) -> String {
    match key_on(layer, row, col) {
        Key::Char(c) => c.to_string(),
        key => bottom_label(key, layer).to_string(),
    }
}

/// Where the keyboard's pieces go on the screen, in art pixels.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct KeyboardPlace {
    /// The modal, `[x, y, w, h]`: where the window face goes.
    pub modal: [u32; 4],
    /// Its one panel, `[x, y, w, h]` in the modal's art pixels.
    pub panel: [u32; 4],
    /// The panel's well inside, `[x, y, w, h]`.
    pub well: [u32; 4],
    /// The field and the keys, `[x, y, w, h]`.
    pub field: [u32; 4],
    pub keys: [u32; 4],
    /// The hint line's top left.
    pub hints: [u32; 2],
}

/// Where the keyboard goes on this canvas: the modal at `modal_rect()`, its panel filling it,
/// and the field, the keys and the hint line centred in the panel's well, each offset rounded
/// down to a whole art pixel as flow.html rounds them.
pub fn keyboard_place() -> KeyboardPlace {
    let modal = modal_rect();
    let [mx, my, mw, mh] = modal;
    let panel = modal_layout(mw, mh, &[1])[0];
    let [px, py, pw, ph] = panel;
    let well = [
        mx + px + WELL_X,
        my + py + WELL_Y,
        pw.saturating_sub(2 * WELL_X),
        ph.saturating_sub(PANEL_CHROME_H),
    ];
    let x = well[0] + well[2].saturating_sub(KEYS_W) / 2;
    let y = well[1] + well[3].saturating_sub(STACK_H) / 2;
    let keys_y = y + FIELD_H + STACK_GAP;
    let hints_w = hint_chars() * ADVANCE as u32;
    KeyboardPlace {
        modal,
        panel,
        well,
        field: [x, y, KEYS_W, FIELD_H],
        keys: [x, keys_y, KEYS_W, KEYS_H],
        // centred by its ink, which is a column narrower than its letters
        hints: [
            x + (KEYS_W + 1).saturating_sub(hints_w) / 2,
            keys_y + KEYS_H + STACK_GAP,
        ],
    }
}

/// How many letters the hint line is.
fn hint_chars() -> u32 {
    let words: usize = HINTS.iter().map(|(b, d)| b.len() + 1 + d.len()).sum();
    (words + HINT_GAP * (HINTS.len() - 1)) as u32
}

/// The panel's title for `prompt`, in capitals as the kit's titles are, cut short with "..."
/// where it would run the title tab past a panel `panel_w` wide.
pub fn keyboard_title(prompt: &str, panel_w: u32) -> String {
    let room = panel_w.saturating_sub(TITLE_ROOM);
    let mut title = prompt.to_uppercase();
    if text_width(&title) <= room {
        return title;
    }
    while !title.is_empty() && text_width(&format!("{}...", title.trim_end())) > room {
        title.pop();
    }
    format!("{}...", title.trim_end())
}

/// `src` laid over `dst` with its top left at (`x`, `y`): what is opaque in it replaces what is
/// under it, what is clear leaves it, and what falls outside `dst` is dropped.
fn over(dst: &mut UndoFace, src: &UndoFace, x: u32, y: u32) {
    for sy in 0..src.h {
        for sx in 0..src.w {
            let (dx, dy) = (x + sx, y + sy);
            let from = ((sy * src.w + sx) * 4) as usize;
            if dx >= dst.w || dy >= dst.h || src.rgba[from + 3] == 0 {
                continue;
            }
            let to = ((dy * dst.w + dx) * 4) as usize;
            dst.rgba[to..to + 4].copy_from_slice(&src.rgba[from..from + 4]);
        }
    }
}

/// The modal, its panel titled with `prompt` and the hint line under where the keys go: what
/// the keyboard shows that only its prompt and the palette change. `KeyboardPlace::modal`'s
/// size, drawn at its top left.
pub fn window_face(prompt: &str, pal: &Palette) -> UndoFace {
    let place = keyboard_place();
    let [mx, my, mw, mh] = place.modal;
    let mut face = modal_face(mw, mh, pal);
    let [px, py, pw, ph] = place.panel;
    let panel = panel_face(pw, ph, &keyboard_title(prompt, pw), true, pal);
    over(&mut face, &panel.face, px, py);

    // each button in the frame's bright colour, what it does dim
    let [hx, hy] = place.hints;
    let mut x = hx - mx;
    for (button, does) in HINTS {
        over(&mut face, &text_face(button, pal.frame_hi), x, hy - my);
        x += (button.len() + 1) as u32 * ADVANCE as u32;
        over(&mut face, &text_face(does, pal.dim), x, hy - my);
        x += (does.len() + HINT_GAP) as u32 * ADVANCE as u32;
    }
    face
}

/// A key `rect` of `face` drawn with its `label` centred by its ink, n letters being 6n - 1
/// pixels of it: cream on the tab colour, or dark on the bright face when `lit`.
fn key_into(face: &mut UndoFace, [x, y, w, h]: [u32; 4], label: &str, lit: bool, pal: &Palette) {
    let sprite = mirror(if lit { &KEY_LIT } else { &KEY });
    let rect = [x as i32, y as i32, w as i32, h as i32];
    nine_slice(face, &sprite, rect, [1; 4], [1; 4], pal);
    let text = text_face(label, if lit { pal.dark } else { pal.cream });
    let ink_w = text.w.saturating_sub(1);
    // inside the outline, the label's line 3 pixels down so its capitals sit 5 clear of it
    let pad = (w - 2).saturating_sub(ink_w) / 2;
    over(face, &text, x + 1 + pad, y + 1 + 3);
}

/// Every key of `layer` as one face, `KEYS_W` by `KEYS_H`, clear between the keys.
pub fn keys_face(layer: Layer, pal: &Palette) -> UndoFace {
    let mut face = blank(KEYS_W, KEYS_H);
    for row in 0..KEY_ROWS {
        for col in 0..row_len(row) {
            let label = key_label(layer, row, col);
            key_into(&mut face, key_box(row, col), &label, false, pal);
        }
    }
    face
}

/// The key at `row`, `col` of `layer` lit, a face its size to draw over it.
pub fn lit_key_face(layer: Layer, row: usize, col: usize, pal: &Palette) -> UndoFace {
    let [_, _, w, h] = key_box(row, col);
    let mut face = blank(w, h);
    key_into(
        &mut face,
        [0, 0, w, h],
        &key_label(layer, row, col),
        true,
        pal,
    );
    face
}

/// The field: what has been typed in cream, the end of it if it is too long, with the cursor
/// after it, a bar the capitals' height past the last letter's blank column; in a dark box with
/// an edge round it, the edge bright while what was typed is refused.
pub fn field_face(typed: &str, refused: bool, pal: &Palette) -> UndoFace {
    let mut face = blank(KEYS_W, FIELD_H);
    let edge = if refused { pal.frame_hi } else { pal.edge };
    fill(&mut face, [0, 0, KEYS_W as i32, FIELD_H as i32], edge);
    let inside = [1, 1, KEYS_W as i32 - 2, FIELD_H as i32 - 2];
    fill(&mut face, inside, pal.dark);
    let skip = typed.chars().count().saturating_sub(FIELD_CHARS);
    let shown: String = typed.chars().skip(skip).collect();
    let [x, y] = FIELD_TEXT;
    let mut cursor = x;
    if !shown.is_empty() {
        let text = text_face(&shown, pal.cream);
        over(&mut face, &text, x, y);
        cursor += text.w;
    }
    // the capitals are rows 2 to 8 of the line
    let bar = [cursor as i32, y as i32 + 2, 1, CAP as i32];
    fill(&mut face, bar, pal.frame_hi);
    face
}

/// What each of the keyboard's faces is baked from. A sync keeps the one it baked last and
/// bakes again only the faces whose part of it has changed since.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KeyboardLook {
    /// The kit palette's backdrop colour, as `palette()` takes it.
    pub colour: u8,
    /// What the panel's title asks for.
    pub prompt: String,
    /// What has been typed, and whether it is being refused.
    pub text: String,
    pub refused: bool,
    /// The layer showing and the key under the cursor.
    pub layer: Layer,
    pub row: usize,
    pub col: usize,
}

/// Which of the keyboard's faces to bake again.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct KeyboardStale {
    pub window: bool,
    /// The three layers' keys.
    pub keys: bool,
    pub field: bool,
    pub lit: bool,
}

impl KeyboardStale {
    pub const ALL: KeyboardStale = KeyboardStale {
        window: true,
        keys: true,
        field: true,
        lit: true,
    };

    pub fn any(self) -> bool {
        self.window || self.keys || self.field || self.lit
    }
}

impl KeyboardLook {
    pub fn new(prompt: &str, keys: &Keyboard, refused: bool, colour: u8) -> Self {
        KeyboardLook {
            colour,
            prompt: prompt.to_string(),
            text: keys.text.clone(),
            refused,
            layer: keys.layer,
            row: keys.row,
            col: keys.col,
        }
    }

    /// What to bake to show `self` where `baked` was baked last: everything the first time and
    /// for a new palette; else the window for a new prompt, the field for new text, and the lit
    /// key when the cursor or the layer moves. The layers' keys only take a new palette.
    pub fn stale(&self, baked: Option<&KeyboardLook>) -> KeyboardStale {
        let Some(b) = baked.filter(|b| b.colour == self.colour) else {
            return KeyboardStale::ALL;
        };
        KeyboardStale {
            window: b.prompt != self.prompt,
            keys: false,
            field: (&b.text, b.refused) != (&self.text, self.refused),
            lit: (b.layer, b.row, b.col) != (self.layer, self.row, self.col),
        }
    }
}

pub type Face = (TexId, u32, u32);

/// The keyboard's faces, uploaded: each a texture and its size in art pixels.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct KeyboardFaces {
    pub window: Face,
    pub field: Face,
    /// Each layer's keys, in [`Layer::ALL`]'s order.
    pub keys: [Face; 3],
    /// The key under the cursor, lit.
    pub lit: Face,
}

pub struct KeyboardScreen<'a> {
    pub keys: &'a Keyboard,
    pub faces: Option<&'a KeyboardFaces>,
}

impl KeyboardScreen<'_> {
    /// The keyboard over whatever is already drawn, at 2x: the window, the field, the keys of
    /// the layer showing, and the key under the cursor lit over them.
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let Some(faces) = self.faces else {
            return;
        };
        let place = keyboard_place();
        let at = |(tex, w, h): Face, x: u32, y: u32| quad(tex, w, h, screen_px(x), screen_px(y));
        out.push(at(faces.window, place.modal[0], place.modal[1]));
        out.push(at(faces.field, place.field[0], place.field[1]));
        let [kx, ky, ..] = place.keys;
        out.push(at(faces.keys[self.keys.layer.index()], kx, ky));
        let [x, y, ..] = key_box(self.keys.row, self.keys.col);
        out.push(at(faces.lit, kx + x, ky + y));
    }
}
