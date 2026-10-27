//! The bar's faces: both pairs of actions, made again when the palette or the font changes, and
//! the hints between each pair, made again when they change or the actions are made again, since
//! they are centred in the room the actions leave. Each is small and made only then, so a slide
//! that swaps the actions uploads nothing once both sets of hints are made.

use cartridge_gfx::{out_w, Compositor, TexId};
use cartridge_ui::kit::action::{action_pair, Font, Side};
use cartridge_ui::kit::bar::{bar_layout, hints_face, Hint};
use cartridge_ui::SPRITE_PX;

use crate::app::App;
use crate::frontend::faces::upload_nearest;
use crate::screens::bar::{Actions, BarSet};

/// The bar's textures, each reused as what it shows changes, and what each was made for.
#[derive(Default)]
pub(crate) struct BarTexes {
    /// Each pair of actions' faces, left then right.
    actions: [[Option<TexId>; 2]; Actions::ALL.len()],
    /// The palette colour and the font the actions were made in.
    actions_built: Option<(u8, Font)>,
    hints: [Option<TexId>; Actions::ALL.len()],
    /// The palette colour and the hints each face between a pair was made with.
    hints_built: [Option<(u8, Vec<Hint<'static>>)>; Actions::ALL.len()],
}

pub(crate) fn sync_bar(app: &mut App, compositor: &mut Compositor, texes: &mut BarTexes) {
    let (colour, font) = (app.palette_colour(), app.font());
    let pal = app.palette();
    if texes.actions_built != Some((colour, font)) {
        let width = out_w() / SPRITE_PX;
        let sets = Actions::ALL.map(|actions| {
            let [left, right] = action_pair(
                actions.action(Side::Left),
                actions.action(Side::Right),
                font,
                pal,
            );
            let held = &mut texes.actions[actions.index()];
            let mut up = |i: usize, face: &cartridge_ui::UndoFace| {
                let tex = upload_nearest(compositor, &mut held[i], face);
                (tex, face.w, face.h)
            };
            BarSet {
                layout: bar_layout(width, &left, &right),
                left: up(0, &left.face),
                right: up(1, &right.face),
            }
        });
        app.set_bar_sets(sets);
        texes.actions_built = Some((colour, font));
        texes.hints_built = Default::default();
    }
    let Some(place) = app.bar_place() else {
        return;
    };
    let at = place.actions.index();
    let want = (colour, app.bar_hints_shown());
    let made = app.bar_faces().is_some_and(|f| f.hints[at].is_some());
    if made && texes.hints_built[at].as_ref() == Some(&want) {
        return;
    }
    let Some(set) = app.bar_faces().map(|f| f.sets[at]) else {
        return;
    };
    let face = hints_face(set.layout.hints[2], &want.1, pal);
    let tex = upload_nearest(compositor, &mut texes.hints[at], &face);
    app.set_bar_hints_face(place.actions, tex);
    texes.hints_built[at] = Some(want);
}
