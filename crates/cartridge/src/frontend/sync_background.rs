//! The picture of the pattern the shelf is drawn over, and the THEME panel.

use cartridge_gfx::{Compositor, TexId};

use crate::app::App;
use crate::frontend::faces::upload;
use crate::frontend::lone::Baked;
use crate::list_view::ListView;
use crate::screens::background::background_face;

/// The picture of the pattern the shelf is drawn over, made again when the style or colour
/// changes.
#[derive(Default)]
pub(crate) struct BackdropTex {
    tex: Option<TexId>,
    built: Option<(cartridge_store::BackdropStyle, u8)>,
}

/// The panel is baked again when its rows or the palette change: a new colour re-tints it, and
/// a new font shows on its FONT row, in the frame it changes in.
pub(crate) fn sync_background(
    app: &mut App,
    compositor: &mut Compositor,
    pattern: &mut BackdropTex,
    panel: &mut Baked<(ListView, u8)>,
) {
    let want = app.backdrop_pattern();
    if pattern.built != want {
        if let Some((style, colour)) = want {
            if let Some(face) = cartridge_ui::backdrop_pattern_face(style, colour) {
                let w = face.w;
                let tex = upload(compositor, &mut pattern.tex, face);
                app.set_backdrop_face(tex, w, style, colour);
            }
        }
        pattern.built = want;
    }
    let Some(view) = app.background_view() else {
        return;
    };
    let pal = app.palette();
    let from = (view, app.palette_colour());
    if let Some(baked) = panel.bake(compositor, from, |(view, _)| background_face(view, pal)) {
        app.set_background_panel(baked);
    }
}
