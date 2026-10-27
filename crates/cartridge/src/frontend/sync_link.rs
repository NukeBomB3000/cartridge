//! The link screen's faces, made while it is open: the art, made again when the palette changes;
//! the LINK panel, when what it shows or the palette does; the bar's actions, when the palette or
//! the font does; and the hints between picking's actions, when they or the actions change. All
//! are small and drawn in a frame, so they are made in the frame the screen opens in.

use cartridge_gfx::{out_w, Compositor, TexId};
use cartridge_ui::kit::action::{action_pair, Font, Side};
use cartridge_ui::kit::bar::{bar_layout, hints_face, Hint};
use cartridge_ui::link_art::{link_art, RINGS};
use cartridge_ui::{UndoFace, SPRITE_PX};

use crate::app::App;
use crate::frontend::faces::upload_nearest;
use crate::frontend::lone::Baked;
use crate::link_screen::{
    link_panel_face, LinkActions, LinkBar, LinkSprites, LinkView, Sprite, PORT_UP,
};
use crate::screens::bar::BarSet;

/// How many faces the art has: two plugs, the cable, the adapter, the port, the arcs either side
/// and the clicks.
const ART: usize = 6 + 2 * RINGS;

/// The link screen's textures, each reused as what it shows changes, and what each was made for.
#[derive(Default)]
pub(crate) struct LinkTexes {
    art: [Option<TexId>; ART],
    /// The palette colour the art was made in.
    art_built: Option<u8>,
    panel: Baked<(LinkView, u8)>,
    actions: [[Option<TexId>; 2]; LinkActions::ALL.len()],
    actions_built: Option<(u8, Font)>,
    hints: Option<TexId>,
    hints_built: Option<(u8, Vec<Hint<'static>>)>,
}

pub(crate) fn sync_link(app: &mut App, compositor: &mut Compositor, texes: &mut LinkTexes) {
    let Some(view) = app.link_view() else {
        return;
    };
    let (colour, font, pal) = (app.palette_colour(), app.font(), app.palette());
    if texes.art_built != Some(colour) {
        let art = link_art(pal, out_w() / SPRITE_PX, PORT_UP);
        let faces: [&UndoFace; ART] = [
            &art.plug_host,
            &art.plug_join,
            &art.cable,
            &art.adapter,
            &art.port,
            &art.arcs_right[0],
            &art.arcs_right[1],
            &art.arcs_right[2],
            &art.arcs_left[0],
            &art.arcs_left[1],
            &art.arcs_left[2],
            &art.clicks,
        ];
        let mut up = faces
            .iter()
            .zip(texes.art.iter_mut())
            .map(|(face, held)| Sprite {
                tex: upload_nearest(compositor, held, face),
                w: face.w,
                h: face.h,
            });
        let mut next = || up.next().expect("a texture for every face");
        let sprites = LinkSprites {
            plug_host: next(),
            plug_join: next(),
            cable: next(),
            adapter: next(),
            port: next(),
            arcs_right: [next(), next(), next()],
            arcs_left: [next(), next(), next()],
            clicks: next(),
        };
        app.set_link_sprites(sprites);
        texes.art_built = Some(colour);
    }
    if let Some(panel) = texes.panel.bake(compositor, (view, colour), |(view, _)| {
        link_panel_face(view, pal)
    }) {
        app.set_link_panel(panel);
    }
    let width = out_w() / SPRITE_PX;
    if texes.actions_built != Some((colour, font)) {
        let sets = LinkActions::ALL.map(|actions| {
            let [left, right] = action_pair(
                actions.action(Side::Left),
                actions.action(Side::Right),
                font,
                pal,
            );
            let held = &mut texes.actions[actions.index()];
            let mut up = |i: usize, face: &UndoFace| {
                (
                    upload_nearest(compositor, &mut held[i], face),
                    face.w,
                    face.h,
                )
            };
            BarSet {
                layout: bar_layout(width, &left, &right),
                left: up(0, &left.face),
                right: up(1, &right.face),
            }
        });
        app.set_link_bar(LinkBar { sets, hints: None });
        texes.actions_built = Some((colour, font));
        texes.hints_built = None;
    }
    let want = (colour, app.link_hints());
    if texes.hints_built.as_ref() == Some(&want) {
        return;
    }
    let Some(mut bar) = app.link_bar else {
        return;
    };
    let room = bar.sets[LinkActions::Pick.index()].layout.hints[2];
    let face = hints_face(room, &want.1, pal);
    bar.hints = Some(upload_nearest(compositor, &mut texes.hints, &face));
    app.set_link_bar(bar);
    texes.hints_built = Some(want);
}
