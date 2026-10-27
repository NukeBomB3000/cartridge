//! What the screens' syncs share: the uploads that reuse a texture as what it shows changes.

use cartridge_gfx::{Compositor, TexId};

pub(crate) fn upload(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    face: cartridge_ui::UndoFace,
) -> TexId {
    upload_rgba(compositor, held, face.w, face.h, &face.rgba)
}

pub(crate) fn upload_rgba(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    w: u32,
    h: u32,
    rgba: &[u8],
) -> TexId {
    match *held {
        Some(id) => {
            compositor.update_texture(id, w, h, rgba);
            id
        }
        None => {
            let id = compositor.create_texture(w, h, rgba);
            *held = Some(id);
            id
        }
    }
}

/// A kit face (one texel to an art pixel, drawn at 2x by `cartridge_ui::kit::pixel::quad`) into the
/// texture `held`, made on first use with NEAREST filtering so it stays sharp at 2x, and filled
/// in place after that (an updated texture keeps its filter).
pub(crate) fn upload_nearest(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    face: &cartridge_ui::UndoFace,
) -> TexId {
    match *held {
        Some(id) => {
            compositor.update_texture(id, face.w, face.h, &face.rgba);
            id
        }
        None => {
            let id = compositor.create_texture_nearest(face.w, face.h, &face.rgba);
            *held = Some(id);
            id
        }
    }
}
