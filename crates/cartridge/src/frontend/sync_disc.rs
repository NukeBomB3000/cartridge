//! The DISC screen's faces, made while it is open: the DISC panel, when what it shows or the
//! palette does; the bar's actions, when the palette or the font does; and the hint between
//! them, with the actions. All are small and drawn in the frame the screen opens in.

use cartridge_gfx::{out_w, Compositor, TexId};
use cartridge_ui::kit::action::{action_pair, Font, Side};
use cartridge_ui::kit::bar::{bar_layout, hints_face};
use cartridge_ui::{UndoFace, SPRITE_PX};

use crate::app::App;
use crate::disc_screen::{disc_action, disc_panel_face, DiscBar, DiscView, PICK};
use crate::frontend::faces::upload_nearest;
use crate::frontend::lone::Baked;
use crate::screens::bar::BarSet;

/// The DISC screen's textures, each reused as what it shows changes, and what each was made for.
#[derive(Default)]
pub(crate) struct DiscTexes {
    panel: Baked<(DiscView, u8)>,
    actions: [Option<TexId>; 2],
    hints: Option<TexId>,
    /// The palette colour and the font the bar was made in.
    built: Option<(u8, Font)>,
}

pub(crate) fn sync_disc(app: &mut App, compositor: &mut Compositor, texes: &mut DiscTexes) {
    let Some(view) = app.disc_view() else {
        return;
    };
    let (colour, font, pal) = (app.palette_colour(), app.font(), app.palette());
    if let Some(panel) = texes.panel.bake(compositor, (view, colour), |(view, _)| {
        disc_panel_face(view, pal)
    }) {
        app.set_disc_panel(panel);
    }
    if texes.built == Some((colour, font)) {
        return;
    }
    let [left, right] = action_pair(disc_action(Side::Left), disc_action(Side::Right), font, pal);
    let layout = bar_layout(out_w() / SPRITE_PX, &left, &right);
    let mut up = |i: usize, face: &UndoFace| {
        (
            upload_nearest(compositor, &mut texes.actions[i], face),
            face.w,
            face.h,
        )
    };
    let set = BarSet {
        layout,
        left: up(0, &left.face),
        right: up(1, &right.face),
    };
    let hints = hints_face(layout.hints[2], &[PICK], pal);
    let hints = upload_nearest(compositor, &mut texes.hints, &hints);
    app.set_disc_bar(DiscBar {
        set,
        hints: Some(hints),
    });
    texes.built = Some((colour, font));
}
