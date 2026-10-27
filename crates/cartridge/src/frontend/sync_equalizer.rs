//! The EQUALIZER panel, baked again when a band, the cursor, the test sound or the palette
//! changes.

use cartridge_gfx::Compositor;

use crate::app::App;
use crate::equalizer_screen::EqView;
use crate::frontend::lone::Baked;
use crate::screens::equalizer::equalizer_face;

pub(crate) fn sync_equalizer(
    app: &mut App,
    compositor: &mut Compositor,
    panel: &mut Baked<(EqView, u8)>,
) {
    let Some(view) = app.equalizer_view() else {
        return;
    };
    let pal = app.palette();
    let from = (view, app.palette_colour());
    if let Some(baked) = panel.bake(compositor, from, |(view, _)| equalizer_face(view, pal)) {
        app.set_equalizer_panel(baked);
    }
}
