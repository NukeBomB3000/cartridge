//! The on-screen keyboard's faces, in the kit palette: its window, its field, every layer's
//! keys and the lit key, each baked again only when what it shows changes.

use cartridge_gfx::{Compositor, TexId};
use cartridge_ui::{
    field_face, keys_face, lit_key_face, window_face, KeyboardFaces, KeyboardLook, Layer, UndoFace,
};

use crate::app::App;
use crate::frontend::faces::upload_nearest;

/// The keyboard's textures, each filled again in place, and what they were last baked from.
#[derive(Default)]
pub(crate) struct KeyboardTexes {
    window: Option<TexId>,
    field: Option<TexId>,
    keys: [Option<TexId>; 3],
    lit: Option<TexId>,
    baked: Option<(KeyboardLook, KeyboardFaces)>,
}

pub(crate) fn sync_keyboard(app: &mut App, compositor: &mut Compositor, held: &mut KeyboardTexes) {
    let Some(typing) = app.typing() else {
        return;
    };
    let look = KeyboardLook::new(
        &typing.prompt,
        typing.keys,
        typing.refused,
        app.palette_colour(),
    );
    let stale = look.stale(held.baked.as_ref().map(|(look, _)| look));
    if !stale.any() {
        return;
    }
    let pal = app.palette();
    let mut up = |tex: &mut Option<TexId>, face: UndoFace| {
        (upload_nearest(compositor, tex, &face), face.w, face.h)
    };
    // what was baked before and still shows the same
    let kept = held.baked.as_ref().map(|(_, faces)| *faces);
    let keep = |still: bool| kept.filter(|_| still);
    let faces = KeyboardFaces {
        keys: match keep(!stale.keys) {
            Some(f) => f.keys,
            None => Layer::ALL.map(|l| up(&mut held.keys[l.index()], keys_face(l, pal))),
        },
        window: match keep(!stale.window) {
            Some(f) => f.window,
            None => up(&mut held.window, window_face(&look.prompt, pal)),
        },
        field: match keep(!stale.field) {
            Some(f) => f.field,
            None => up(&mut held.field, field_face(&look.text, look.refused, pal)),
        },
        lit: match keep(!stale.lit) {
            Some(f) => f.lit,
            None => up(
                &mut held.lit,
                lit_key_face(look.layer, look.row, look.col, pal),
            ),
        },
    };
    held.baked = Some((look, faces));
    app.set_keyboard_faces(faces);
}
