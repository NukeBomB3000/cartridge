//! The shelf's system (or the letter jumped to) in the bar's pill, the alert a refused cart comes
//! out with, and the Date & Time panel. The top bar prints its clock and the battery's charge in
//! its own ink (sync_top_bar.rs), over the shelf and over the save-state switcher alike.

use omega_gfx::{Compositor, TexId};
use omega_ui::kit::action::pill_face;
use omega_ui::{alert_face, Field};

use crate::app::App;
use crate::frontend::faces::upload_nearest;
use crate::frontend::lone::Baked;
use crate::screens::clock::clock_face;

#[derive(Default)]
pub(crate) struct Clocks {
    /// The Date & Time panel, baked again when the picker's line, its cursor or the palette
    /// changes.
    panel: Baked<(String, Field, u8)>,
    /// What the pill shows and the palette colour it was baked in, and its texture.
    platform: (String, u8),
    platform_tex: Option<TexId>,
    /// The selected game's name under the row and the palette colour it was baked in.
    title: (String, u8),
    title_tex: Option<TexId>,
    /// The refused cart's alert, baked again when the palette changes.
    alert: Baked<u8>,
}

pub(crate) fn sync_clock(app: &mut App, compositor: &mut Compositor, clocks: &mut Clocks) {
    if let Some(picker) = app.picker() {
        let pal = app.palette();
        let from = (picker.line(), picker.cursor(), app.palette_colour());
        let baked = clocks
            .panel
            .bake(compositor, from, |_| clock_face(picker, pal));
        if let Some(panel) = baked {
            app.set_clock_panel(panel);
        }
    }
    let colour = app.palette_colour();
    if let Some(alert) = clocks
        .alert
        .bake(compositor, colour, |_| alert_face(app.palette()))
    {
        app.set_alert_face(alert.tex);
    }
    let named = (app.pill_text().unwrap_or_default(), colour);
    if named != clocks.platform {
        if named.0.is_empty() {
            app.clear_shelf_platform();
        } else {
            let face = pill_face(&named.0, app.palette());
            let id = upload_nearest(compositor, &mut clocks.platform_tex, &face);
            app.set_shelf_platform_face(id, face.w);
        }
        clocks.platform = named;
    }
    let title = (app.selected_title().unwrap_or_default(), colour);
    if title != clocks.title {
        if title.0.is_empty() {
            app.clear_shelf_title();
        } else {
            let face = omega_ui::kit::text::text_face(&title.0, app.palette().cream);
            let id = upload_nearest(compositor, &mut clocks.title_tex, &face);
            app.set_shelf_title_face(id, face.w);
        }
        clocks.title = title;
    }
}
