//! The POWER menu's panel, baked again when its cursor moves or the palette changes, and the
//! panel shown after its choice, baked once for the choice and the palette. Each keeps its own
//! texture, so the menu still has its panel as it slides away from the screen after it.

use cartridge_gfx::Compositor;
use cartridge_ui::PowerChoice;

use crate::app::App;
use crate::frontend::lone::Baked;
use crate::screens::power::{going_face, power_menu_face, PowerScreen};

#[derive(Default)]
pub(crate) struct PowerPanels {
    menu: Baked<(PowerChoice, u8)>,
    going: Baked<(PowerChoice, u8)>,
}

pub(crate) fn sync_power(app: &mut App, compositor: &mut Compositor, held: &mut PowerPanels) {
    let (pal, colour) = (app.palette(), app.palette_colour());
    match app.power_screen() {
        Some(PowerScreen::Menu(row)) => {
            let baked = held.menu.bake(compositor, (row, colour), |&(row, _)| {
                power_menu_face(row, pal)
            });
            if let Some(panel) = baked {
                app.set_power_menu_panel(panel);
            }
        }
        Some(PowerScreen::Going(choice)) => {
            let baked = held
                .going
                .bake(compositor, (choice, colour), |&(choice, _)| {
                    going_face(choice, pal)
                });
            if let Some(panel) = baked {
                app.set_going_panel(panel);
            }
        }
        None => {}
    }
}
