//! The faces of the panel Y opens from a Network row, on textures of their own so the Network
//! modal it slides against keeps its own: the panel baked for its title, the palette and how
//! many lines it holds, each line and the note again only when what it says changes.

use cartridge_gfx::Compositor;
use cartridge_ui::kit::row::{row_face, row_width, Row};
use cartridge_ui::kit::text::text_face;
use cartridge_ui::network::{info_chrome, info_rect, InfoFaces};

use crate::app::App;
use crate::frontend::sync_network::{room_for, Held};
use crate::screens::info::InfoView;

#[derive(Default)]
pub(crate) struct InfoTexes {
    chrome: Held<(&'static str, u8, usize)>,
    well: [u32; 4],
    lines: Vec<Held<(&'static str, String, u8)>>,
    note: Held<(&'static str, u8)>,
    /// What the faces were last made for.
    shown: Option<(InfoView, u8)>,
}

pub(crate) fn sync_info(app: &mut App, compositor: &mut Compositor, state: &mut InfoTexes) {
    let Some(view) = app.info_view() else {
        return;
    };
    let colour = app.palette_colour();
    let now = (view, colour);
    if state.shown.as_ref() == Some(&now) {
        return;
    }
    let (view, colour) = now;
    let pal = app.palette();

    let count = view.lines.len();
    let well = &mut state.well;
    let chrome = state
        .chrome
        .face(compositor, (view.title, colour, count), |&(title, _, n)| {
            let panel = info_chrome(title, n, pal);
            *well = panel.well;
            panel.face
        });
    let width = row_width(state.well[2]);
    room_for(&mut state.lines, count);
    let lines = view
        .lines
        .iter()
        .zip(&mut state.lines)
        .map(|(&(label, ref value), held)| {
            held.face(
                compositor,
                (label, value.clone(), colour),
                |(label, value, _)| {
                    let row = Row {
                        label,
                        value: Some(value),
                        ..Row::default()
                    };
                    row_face(width, &row, pal)
                },
            )
        })
        .collect();
    let note = state
        .note
        .face(compositor, (view.note, colour), |(note, _)| {
            text_face(note, pal.dim)
        });

    app.set_info_faces(InfoFaces {
        chrome,
        at: info_rect(count),
        well: state.well,
        lines,
        note: Some(note),
    });
    state.shown = Some((view, colour));
}
