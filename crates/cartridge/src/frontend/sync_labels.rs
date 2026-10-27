//! The LABELS panel, baked again whenever what it shows or the palette changes.

use cartridge_gfx::Compositor;

use crate::app::App;
use crate::frontend::lone::Baked;
use crate::list_view::ListView;
use crate::screens::labels::labels_face;

pub(crate) fn sync_labels(
    app: &mut App,
    compositor: &mut Compositor,
    panel: &mut Baked<(ListView, u8)>,
) {
    let Some(view) = app.labels_view() else {
        return;
    };
    let pal = app.palette();
    let from = (view, app.palette_colour());
    if let Some(baked) = panel.bake(compositor, from, |(view, _)| labels_face(view, pal)) {
        app.set_labels_panel(baked);
    }
}
