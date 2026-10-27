//! The Network modal and the panels Y opens from its rows, laid out as assets/ui/flow.html lays
//! them out.
//!
//! Network is a modal of two panels: NETWORK, as tall as its rows (Wi-Fi, Network Time, SSH),
//! and AVAILABLE NETWORKS under it, taking the rest of the modal, its networks scrolling a whole
//! row at a time when there are more than it holds, and short notes under them. Each row's own
//! panel (WI-FI, NETWORK TIME, SSH) is a lone panel in the middle of the page, 200 art pixels
//! wide, as tall as its lines and its note.
//!
//! Everything here is in art pixels (two screen pixels each). The faces are the kit's, baked by
//! the frontend at art resolution and drawn at 2x with `kit::pixel::quad`; the modal and its
//! panels are baked as one face, so the frame under the panels is not drawn twice.

use crate::draw::{Draw, TexId};
use crate::kit::layout::{centred, modal_rect, panel_height, screen_px, PANEL_CHROME_H};
use crate::kit::modal::{modal_face, modal_layout};
use crate::kit::palette::Palette;
use crate::kit::panel::{panel_face, title_face, PanelFace};
use crate::kit::pixel::{quad, LINE};
use crate::kit::row::{row_rect, row_width, ROW_H};
use crate::plate::UndoFace;

/// A face uploaded at art resolution: its texture, then its width and height in art pixels.
pub type Face = (TexId, u32, u32);

/// The modal's panels' titles, top first.
pub const NETWORK_TITLE: &str = "NETWORK";
pub const AVAILABLE_TITLE: &str = "AVAILABLE NETWORKS";

/// How wide a row's panel is (flow.css's `.lone.small`).
pub const INFO_W: u32 = 200;

/// A note is text that is not a row (flow.css's `.note`): it starts three pixels into the well,
/// four below the rows over it, one `LINE` to each of its lines. Last in a well it keeps two
/// pixels under its line, alone in one five, so its capitals sit as far from the well's bottom
/// as from its top.
pub const NOTE_IN: u32 = 3;
pub const NOTE_TOP: u32 = 4;
const NOTE_FOOT: u32 = 2;
const NOTE_FOOT_ALONE: u32 = 5;

/// How much of a well `notes` lines of notes take, under rows.
fn notes_room(notes: usize) -> u32 {
    match notes {
        0 => 0,
        n => NOTE_TOP + LINE * n as u32 + NOTE_FOOT,
    }
}

/// The modal's two panels, `[x, y, w, h]` in the modal's art pixels, for a NETWORK panel of
/// `settings` rows: NETWORK as tall as its rows, AVAILABLE NETWORKS a pixel over it, sharing
/// its stripe, and down to the frame.
pub fn network_panels(settings: usize) -> [[u32; 4]; 2] {
    let [_, _, w, h] = modal_rect();
    let [x, y, pw, ph] = modal_layout(w, h, &[1])[0];
    let hug = panel_height(settings as u32 * ROW_H).min(ph);
    [[x, y, pw, hug], [x, y + hug - 1, pw, ph + 1 - hug]]
}

/// How many networks AVAILABLE NETWORKS shows at once, under a NETWORK panel of `settings`
/// rows and over `notes` lines of notes.
pub fn networks_shown(settings: usize, notes: usize) -> usize {
    let [_, [.., h]] = network_panels(settings);
    let well = h.saturating_sub(PANEL_CHROME_H);
    (well.saturating_sub(notes_room(notes)) / ROW_H).max(1) as usize
}

/// The first of `rows` networks to show, `shown` at a time, so the lit one is in view, moving
/// the list as little as it can from where it was: a whole row at a time.
pub fn network_window(lit: usize, rows: usize, was: usize, shown: usize) -> usize {
    if rows <= shown {
        return 0;
    }
    let first = if lit < was {
        lit
    } else if lit >= was + shown {
        lit + 1 - shown
    } else {
        was
    };
    first.min(rows - shown)
}

/// The modal and its panels baked as one face, their titles dim, and where in it each panel's
/// well and title are.
pub struct NetworkChrome {
    pub face: UndoFace,
    /// NETWORK's well then AVAILABLE NETWORKS', `[x, y, w, h]` in the face's art pixels.
    pub wells: [[u32; 4]; 2],
    /// Each panel's title, `[x, y, w, h]` in the face's art pixels, for its `title_face`.
    pub titles: [[u32; 4]; 2],
}

/// The Network modal's chrome for a NETWORK panel of `settings` rows, in `pal`.
pub fn network_chrome(settings: usize, pal: &Palette) -> NetworkChrome {
    let [_, _, w, h] = modal_rect();
    let mut face = modal_face(w, h, pal);
    let mut wells = [[0; 4]; 2];
    let mut titles = [[0; 4]; 2];
    let names = [NETWORK_TITLE, AVAILABLE_TITLE];
    for (i, [x, y, pw, ph]) in network_panels(settings).into_iter().enumerate() {
        let panel = panel_face(pw, ph, names[i], false, pal);
        lay(&mut face, &panel.face, x, y);
        let [wx, wy, ww, wh] = panel.well;
        wells[i] = [x + wx, y + wy, ww, wh];
        let [tx, ty, tw, th] = panel.title;
        titles[i] = [x + tx, y + ty, tw, th];
    }
    NetworkChrome {
        face,
        wells,
        titles,
    }
}

/// Panel `panel`'s title (0 NETWORK, 1 AVAILABLE NETWORKS) as the active panel shows it, for
/// drawing over the chrome's dim one.
pub fn network_title(panel: usize, pal: &Palette) -> UndoFace {
    let title = if panel == 0 {
        NETWORK_TITLE
    } else {
        AVAILABLE_TITLE
    };
    title_face(title, true, pal)
}

/// How wide each panel's rows are: as wide as its well and a pixel more each side, the kit's
/// rule for every row. flow.html's list (in a `.list`) and a panel's lines (in a `.when-online`)
/// miss that pixel, being no panel's own children; the kit's rule wins.
pub fn network_row_widths(wells: &[[u32; 4]; 2]) -> [u32; 2] {
    wells.map(|[_, _, w, _]| row_width(w))
}

/// The Network modal's faces, and where its chrome put each panel.
pub struct NetworkFaces {
    pub chrome: Face,
    /// The chrome's wells and titles, as `NetworkChrome` has them.
    pub wells: [[u32; 4]; 2],
    pub title_at: [[u32; 4]; 2],
    /// Each panel's title lit, for the panel the cursor is in.
    pub titles: [Face; 2],
    /// Which panel the cursor is in: 0 NETWORK, 1 AVAILABLE NETWORKS.
    pub active: usize,
    /// NETWORK's rows, top first.
    pub settings: Vec<Face>,
    /// The networks in view, top first.
    pub networks: Vec<Face>,
    /// The notes under them, a line each.
    pub notes: Vec<Face>,
}

impl NetworkFaces {
    /// The modal over whatever is already drawn: the backdrop.
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let [mx, my, _, _] = modal_rect();
        push(out, self.chrome, mx, my);
        let [tx, ty, _, _] = self.title_at[self.active.min(1)];
        push(out, self.titles[self.active.min(1)], mx + tx, my + ty);
        for (well, rows) in self.wells.iter().zip([&self.settings, &self.networks]) {
            for (i, row) in rows.iter().enumerate() {
                let [x, y, _, _] = row_rect(*well, i as u32);
                push(out, *row, mx + x, my + y);
            }
        }
        let [nx, ny, _, _] = self.wells[1];
        let below = ny + self.networks.len() as u32 * ROW_H + NOTE_TOP;
        for (i, note) in self.notes.iter().enumerate() {
            push(out, *note, mx + nx + NOTE_IN, my + below + i as u32 * LINE);
        }
    }
}

/// Where a row's panel goes for `lines` lines and a note under them: `lines` rows, then the
/// note, or the note on its own.
pub fn info_rect(lines: usize) -> [u32; 4] {
    let foot = if lines == 0 {
        NOTE_FOOT_ALONE
    } else {
        NOTE_FOOT
    };
    let content = lines as u32 * ROW_H + NOTE_TOP + LINE + foot;
    centred(INFO_W, panel_height(content))
}

/// A row's panel titled `title` for `lines` lines, in `pal`: its title lit, as the only panel
/// there is.
pub fn info_chrome(title: &str, lines: usize, pal: &Palette) -> PanelFace {
    let [_, _, w, h] = info_rect(lines);
    panel_face(w, h, title, true, pal)
}

/// A row's panel's faces, for `lines` lines.
pub struct InfoFaces {
    /// The panel, its title lit: it is the only one.
    pub chrome: Face,
    /// The panel's `[x, y, w, h]`, as `info_rect` has it.
    pub at: [u32; 4],
    /// Its well, `[x, y, w, h]` in the panel's art pixels.
    pub well: [u32; 4],
    /// Its lines, top first: SSH's IP, HOST, USER, PASSWORD and PORT, say. None while there is
    /// nothing to tell, the note then saying why.
    pub lines: Vec<Face>,
    pub note: Option<Face>,
}

impl InfoFaces {
    /// The panel over whatever is already drawn: the backdrop.
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let [px, py, _, _] = self.at;
        push(out, self.chrome, px, py);
        for (i, line) in self.lines.iter().enumerate() {
            let [x, y, _, _] = row_rect(self.well, i as u32);
            push(out, *line, px + x, py + y);
        }
        let [wx, wy, _, _] = self.well;
        if let Some(note) = self.note {
            let y = wy + self.lines.len() as u32 * ROW_H + NOTE_TOP;
            push(out, note, px + wx + NOTE_IN, py + y);
        }
    }
}

/// `face` drawn at 2x with its top left at art pixel (`x`, `y`).
fn push(out: &mut Vec<Draw>, (tex, w, h): Face, x: u32, y: u32) {
    out.push(quad(tex, w, h, screen_px(x), screen_px(y)));
}

/// `top` laid over `under` with its top left at (`x`, `y`), wherever it is not clear.
fn lay(under: &mut UndoFace, top: &UndoFace, x: u32, y: u32) {
    for ty in 0..top.h.min(under.h.saturating_sub(y)) {
        for tx in 0..top.w.min(under.w.saturating_sub(x)) {
            let from = ((ty * top.w + tx) * 4) as usize;
            if top.rgba[from + 3] == 0 {
                continue;
            }
            let to = (((y + ty) * under.w + x + tx) * 4) as usize;
            under.rgba[to..to + 4].copy_from_slice(&top.rgba[from..from + 4]);
        }
    }
}
