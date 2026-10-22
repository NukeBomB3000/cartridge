//! The shelf: the carts in a row over the background and the bar across the top. Each is a
//! layer of its own as the shelf slides away to a menu and back. The bar across the bottom, with
//! the shelf's system in its pill, is drawn over every screen (bar.rs).

use omega_ui::{ease, Draw, TopBar};

use crate::app::App;
use crate::nav::Offsets;

const CORE_PICKER_RECEDE: f32 = 0.26;
const CORE_PICKER_DIM: f32 = 0.614;
/// How far above the bottom bar the selected game's name stands in its pill, in art pixels.
const TITLE_GAP: u32 = 4;

impl App {
    /// The shelf, its carts moved `off.carts_dx` across and `off.carts_dy` down, its top bar
    /// `off.bar_dy` down.
    pub(crate) fn draw_shelf(&self, off: Offsets, out: &mut Vec<Draw>) {
        let carts = out.len();
        match (self.core_picker_shown(), self.selected_stem()) {
            (Some(picker), Some(stem)) => {
                let open = ease(picker.openness(self.now()));
                let dim = 1.0 + (CORE_PICKER_DIM - 1.0) * open;
                self.shelf()
                    .draw_row(Some(stem), 0.0, CORE_PICKER_RECEDE * open, dim, out);
                shift(&mut out[carts..], off.carts_dx, off.carts_dy);
            }
            _ => {
                self.shelf()
                    .draw_row(None, self.shelf_shake(), 0.0, 1.0, out);
                shift(&mut out[carts..], off.carts_dx, off.carts_dy);
            }
        }
        // the selected game's name in a pill as the bar's, over the bar, going with the carts; not
        // while the core picker stands over the row
        if let (None, Some(tex)) = (self.core_picker_shown(), self.shelf_title.face) {
            let px = omega_ui::SPRITE_PX;
            let w = self.shelf_title.w;
            let h = omega_ui::kit::action::BAR;
            let x = (omega_gfx::out_w() / px / 2).saturating_sub(w / 2) * px;
            let bar_top = omega_ui::kit::bar::BarLayout::top(omega_gfx::out_h());
            let y = bar_top - ((h + TITLE_GAP) * px) as f32;
            let at = out.len();
            out.push(omega_ui::kit::pixel::quad(tex, w, h, x as f32, y));
            shift(&mut out[at..], off.carts_dx, off.carts_dy);
        }
        let bar = out.len();
        self.draw_top_bar(out);
        shift(&mut out[bar..], 0.0, off.bar_dy);
    }

    /// The bar across the top, in the chrome's colours, its faces baked in its ink.
    pub(crate) fn draw_top_bar(&self, out: &mut Vec<Draw>) {
        let r = self.top_readouts;
        TopBar {
            colours: self.chrome_colours(),
            links: self.links(),
            faces: self.link_faces.as_ref(),
            clock: r.clock,
            battery: self.battery,
            percent: r.percent,
            bolt: r.bolt,
        }
        .draw(out);
    }
}

/// Moves each of `drawn` `dx` across and `dy` down.
fn shift(drawn: &mut [Draw], dx: f32, dy: f32) {
    for d in drawn {
        *d = d.shifted(dx, dy);
    }
}
