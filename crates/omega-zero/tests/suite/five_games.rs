use omega_input::{Action, Btn};
use omega_store::{read_focus, Platform};
use omega_zero::screens::settings::SettingsRow;

use crate::common::{boot, tmp_root_with_carts};
use omega_zero::app::App;

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
    a.finish_slide();
}

/// Settings opened and the cursor down on 5 Games.
fn to_five_games(a: &mut App) {
    press(a, Btn::Start);
    while a.settings_row() != Some(SettingsRow::FiveGames) {
        press(a, Btn::Down);
    }
}

/// One cart along the shelf, the row given time to settle.
fn step_right(a: &mut App) {
    a.apply(Action::ShelfRight);
    a.apply(Action::GbaUp(Btn::Right));
    for _ in 0..60 {
        a.update(1.0 / 60.0);
    }
}

fn on_shelf(a: &App) -> Vec<String> {
    a.carts().map(|c| c.stem.clone()).collect()
}

#[test]
fn games_picked_on_the_shelf_become_the_only_shelf_until_switched_off() {
    let d = tmp_root_with_carts(&["Advance Wars", "Emerald", "Golden Sun", "Metroid"]);
    let mut a = boot(d.path());
    assert!(!a.focus_on());
    to_five_games(&mut a);
    press(&mut a, Btn::A);
    assert_eq!(
        a.picking_count(),
        Some(0),
        "A on 5 Games did not start the picking"
    );
    assert_eq!(a.pill_text().as_deref(), Some("PICK 0/5"));

    let first = a.selected_stem().unwrap().to_string();
    press(&mut a, Btn::A);
    assert!(
        a.selected_title().unwrap().starts_with("+ "),
        "the pick is not marked"
    );
    step_right(&mut a);
    let second = a.selected_stem().unwrap().to_string();
    assert_ne!(first, second);
    press(&mut a, Btn::A);
    assert_eq!(a.picking_count(), Some(2));
    press(&mut a, Btn::Start);

    assert!(a.focus_on() && a.picking_count().is_none());
    assert_eq!(on_shelf(&a), [first.clone(), second.clone()]);
    assert_eq!(
        read_focus(d.path()),
        Some(vec![
            (Platform::Gba, first.clone()),
            (Platform::Gba, second.clone())
        ])
    );

    let mut a = boot(d.path());
    assert_eq!(on_shelf(&a), [first, second], "a restart forgot Five Games");

    to_five_games(&mut a);
    press(&mut a, Btn::A);
    assert!(!a.focus_on());
    assert_eq!(
        on_shelf(&a).len(),
        4,
        "switching off did not bring every game back"
    );
    assert_eq!(read_focus(d.path()), None);
}

#[test]
fn no_more_than_five_are_picked_and_none_is_no_choice() {
    let d = tmp_root_with_carts(&["A1", "A2", "A3", "A4", "A5", "A6"]);
    let mut a = boot(d.path());
    to_five_games(&mut a);
    press(&mut a, Btn::A);
    press(&mut a, Btn::Start);
    assert!(!a.focus_on(), "START with nothing picked switched it on");
    for _ in 0..6 {
        press(&mut a, Btn::A);
        step_right(&mut a);
    }
    assert_eq!(a.picking_count(), Some(5));
    press(&mut a, Btn::B);
    assert!(
        a.picking_count().is_none() && !a.focus_on(),
        "B did not give up"
    );
    assert_eq!(on_shelf(&a).len(), 6);
}
