//! The picture viewer's panel, baked again when what it says or the palette changes.

use cartridge_gfx::Compositor;

use crate::app::App;
use crate::frontend::lone::Baked;
use crate::list_view::ListView;
use crate::screens::picture_view::viewer_face;

pub(crate) fn sync_viewer(
    app: &mut App,
    compositor: &mut Compositor,
    panel: &mut Baked<(ListView, u8)>,
) {
    let Some(view) = app.viewer_view() else {
        return;
    };
    let pal = app.palette();
    let from = (view, app.palette_colour());
    if let Some(baked) = panel.bake(compositor, from, |(view, _)| viewer_face(view, pal)) {
        app.set_viewer_panel(baked);
    }
}
