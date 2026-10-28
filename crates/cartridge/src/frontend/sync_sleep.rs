//! The SLEEP panel, baked again when a row or the palette changes.

use cartridge_gfx::Compositor;

use crate::app::App;
use crate::frontend::lone::Baked;
use crate::list_view::ListView;
use crate::screens::sleep::sleep_face;

pub(crate) fn sync_sleep(
    app: &mut App,
    compositor: &mut Compositor,
    panel: &mut Baked<(ListView, u8)>,
) {
    let Some(view) = app.sleep_view() else {
        return;
    };
    let pal = app.palette();
    let from = (view, app.palette_colour());
    if let Some(baked) = panel.bake(compositor, from, |(view, _)| sleep_face(view, pal)) {
        app.set_sleep_panel(baked);
    }
}
