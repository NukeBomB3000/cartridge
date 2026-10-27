//! A lone panel's texture, and what it was baked from, so its sync bakes and uploads it again
//! only when that changes: what it shows, or the palette it is drawn in.

use cartridge_gfx::{Compositor, TexId};
use cartridge_ui::UndoFace;

use crate::frontend::faces::upload_nearest;
use crate::screens::lone::Lone;

pub(crate) struct Baked<K> {
    tex: Option<TexId>,
    from: Option<K>,
}

impl<K> Default for Baked<K> {
    fn default() -> Self {
        Baked {
            tex: None,
            from: None,
        }
    }
}

impl<K: PartialEq> Baked<K> {
    /// The panel `face` makes from `from`, uploaded into this texture, when it was last baked
    /// from something else; None while it still shows `from`.
    pub(crate) fn bake(
        &mut self,
        compositor: &mut Compositor,
        from: K,
        face: impl FnOnce(&K) -> UndoFace,
    ) -> Option<Lone> {
        if self.from.as_ref() == Some(&from) {
            return None;
        }
        let face = face(&from);
        let tex = upload_nearest(compositor, &mut self.tex, &face);
        self.from = Some(from);
        Some(Lone {
            tex,
            w: face.w,
            h: face.h,
        })
    }
}
