//! The ACHIEVEMENTS panel, baked again when a row, a line under them or the palette changes.

use cartridge_gfx::Compositor;

use crate::app::App;
use crate::frontend::lone::Baked;
use crate::list_view::ListView;
use crate::screens::cheevos::cheevos_face;

pub(crate) fn sync_cheevos(
    app: &mut App,
    compositor: &mut Compositor,
    panel: &mut Baked<(ListView, u8)>,
) {
    let Some(view) = app.cheevos_view() else {
        return;
    };
    let pal = app.palette();
    let from = (view, app.palette_colour());
    if let Some(baked) = panel.bake(compositor, from, |(view, _)| cheevos_face(view, pal)) {
        app.set_cheevos_panel(baked);
    }
}
