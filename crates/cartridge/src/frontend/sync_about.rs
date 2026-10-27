//! The About modal, baked again whenever what it shows (the battery) or the palette changes.

use cartridge_gfx::Compositor;

use crate::app::App;
use crate::frontend::lone::Baked;
use crate::screens::about::{about_face, AboutView};

pub(crate) fn sync_about(
    app: &mut App,
    compositor: &mut Compositor,
    modal: &mut Baked<(AboutView, u8)>,
) {
    let Some(view) = app.about_view() else {
        return;
    };
    let pal = app.palette();
    let from = (view, app.palette_colour());
    if let Some(baked) = modal.bake(compositor, from, |(view, _)| about_face(view, pal)) {
        app.set_about_modal(baked);
    }
}
