//! The CONTROLS panel, baked the first time it opens and again only for another system's shelf
//! or a new palette.

use cartridge_gfx::Compositor;

use crate::app::App;
use crate::frontend::lone::Baked;
use crate::screens::controls::{controls_face, ControlsView};

pub(crate) fn sync_controls(
    app: &mut App,
    compositor: &mut Compositor,
    held: &mut Baked<(ControlsView, u8)>,
) {
    let Some(view) = app.controls_view() else {
        return;
    };
    let pal = app.palette();
    if let Some(panel) = held.bake(compositor, (view, app.palette_colour()), |(view, _)| {
        controls_face(view, pal)
    }) {
        app.set_controls_panel(panel);
    }
}
