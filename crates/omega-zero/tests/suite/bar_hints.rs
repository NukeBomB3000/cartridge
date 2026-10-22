use omega_ui::kit::action::action_pair;
use omega_ui::kit::action::Side;
use omega_ui::kit::bar::{bar_layout, hints_width};
use omega_ui::kit::palette::palette;
use omega_zero::screens::bar::Actions;
use omega_zero::screens::labels::labels_hints;

/// Every set of hints the Labels screen shows fits the room between a menu's actions on the
/// narrowest canvas, 640 pixels across: hints wider than it ran under CONFIRM, cut off.
#[test]
fn the_labels_hints_fit_between_a_menus_actions() {
    use omega_zero::list_view::LegendHint;
    for font in [
        omega_ui::kit::pixel::Font::Bold,
        omega_ui::kit::pixel::Font::Thin,
    ] {
        let pal = palette(0);
        let [left, right] = action_pair(
            Actions::Menu.action(Side::Left),
            Actions::Menu.action(Side::Right),
            font,
            pal,
        );
        let room = bar_layout(640 / omega_ui::SPRITE_PX, &left, &right).hints[2];
        for legend in [
            vec![LegendHint::SignOut, LegendHint::Start],
            vec![LegendHint::SignOut, LegendHint::Stop],
        ] {
            let hints = labels_hints(&legend);
            assert!(
                hints_width(&hints) <= room,
                "{font:?}: {hints:?} are {} wide, the room {room}",
                hints_width(&hints)
            );
        }
    }
}
