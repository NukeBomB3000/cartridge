//! The HUD's faces: the gauge, the toast and the badge in the corner, each drawn into a texture
//! of its own when the HUD first shows it, and again when what it shows or the palette changes.
//! A gauge's face changes only when a cell lights or goes out, so holding rewind or a volume
//! button draws one now and then, not every frame.

use cartridge_gfx::{Compositor, TexId};
use cartridge_ui::kit::palette::Palette;
use cartridge_ui::{HudFace, UndoFace};

use crate::app::App;
use crate::frontend::faces::upload_nearest;

/// A face's texture, and what it was drawn for in which palette colour.
struct Drawn<K> {
    tex: Option<TexId>,
    from: Option<(K, u8)>,
}

impl<K> Default for Drawn<K> {
    fn default() -> Self {
        Drawn {
            tex: None,
            from: None,
        }
    }
}

impl<K: PartialEq + Copy> Drawn<K> {
    /// The face `face` draws of `shown` in `colour`'s palette, uploaded into this texture when it
    /// was last drawn of something else; None while it still shows that.
    fn draw(
        &mut self,
        compositor: &mut Compositor,
        shown: K,
        colour: u8,
        face: impl FnOnce() -> UndoFace,
    ) -> Option<HudFace> {
        if self.from == Some((shown, colour)) {
            return None;
        }
        let face = face();
        let tex = upload_nearest(compositor, &mut self.tex, &face);
        self.from = Some((shown, colour));
        Some(HudFace {
            tex,
            w: face.w,
            h: face.h,
        })
    }
}

/// The HUD's textures, each reused as what it shows changes.
#[derive(Default)]
pub(crate) struct HudTexes {
    gauge: Drawn<cartridge_ui::Gauge>,
    toast: Drawn<cartridge_ui::Toast>,
    badge: Drawn<cartridge_ui::CornerBadge>,
}

pub(crate) fn sync_hud(app: &mut App, compositor: &mut Compositor, texes: &mut HudTexes) {
    let colour = app.palette_colour();
    let pal: &Palette = app.palette();
    let hud = app.hud_mut();
    if let Some(gauge) = hud.gauge() {
        if let Some(face) = texes
            .gauge
            .draw(compositor, gauge, colour, || gauge.face(pal))
        {
            hud.set_gauge_face(gauge, face);
        }
    }
    if let Some(toast) = hud.last_toast() {
        if let Some(face) = texes
            .toast
            .draw(compositor, toast, colour, || toast.face(pal))
        {
            hud.set_toast_face(toast, face);
        }
    }
    if let Some(badge) = hud.badge() {
        if let Some(face) = texes
            .badge
            .draw(compositor, badge, colour, || badge.face(pal))
        {
            hud.set_badge_face(badge, face);
        }
    }
}
