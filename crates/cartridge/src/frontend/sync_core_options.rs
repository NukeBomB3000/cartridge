//! The core's options panel, baked again when a value, the lit row or the palette changes.

use cartridge_gfx::Compositor;

use crate::app::App;
use crate::core_options::core_options_face;
use crate::frontend::lone::Baked;
use crate::list_view::ListView;

pub(crate) fn sync_core_options(
    app: &mut App,
    compositor: &mut Compositor,
    panel: &mut Baked<(String, ListView, u8)>,
) {
    let Some((title, view)) = app.core_options_view() else {
        return;
    };
    let pal = app.palette();
    let from = (title, view, app.palette_colour());
    if let Some(baked) = panel.bake(compositor, from, |(title, view, _)| {
        core_options_face(title, view, pal)
    }) {
        app.set_core_options_panel(baked);
    }
}
