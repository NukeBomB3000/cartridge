use crate::common;

use cartridge::app::Phase;
use cartridge_input::Action;
use cartridge_store::{write_app_state, AppState};
use common::{app_playing_in, boot, tmp_root_with_carts};

#[test]
fn one_cart_boots_straight_into_the_game() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let a = boot(d.path());
    assert!(a.single_cart());
    assert!(
        matches!(a.phase(), Phase::Inserting { .. }),
        "the shelf should be unreachable"
    );
}

#[test]
fn one_cart_boots_even_when_app_state_names_a_cart_that_is_gone() {
    let d = tmp_root_with_carts(&["Emerald"]);
    write_app_state(
        d.path(),
        &AppState {
            cart: Some("Deleted".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let a = boot(d.path());
    assert!(matches!(a.phase(), Phase::Inserting { .. }));
}

#[test]
fn eject_is_refused_with_only_one_cart() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::Eject);
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "it ejected with nowhere to go"
    );
    assert!(a.refusal_active(a.now()), "the held MENU said nothing");
}

#[test]
fn two_carts_still_show_the_shelf_and_still_eject() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let a = boot(d.path());
    assert!(!a.single_cart());
    assert!(matches!(a.phase(), Phase::Shelf));
    let mut a2 = app_playing_in(d.path(), "Emerald");
    a2.apply(Action::Eject);
    assert!(matches!(a2.phase(), Phase::Ejecting { .. }));
}

#[test]
fn a_single_cart_that_will_not_load_falls_back_to_the_shelf() {
    let d = tmp_root_with_carts(&["Broken"]);
    let mut a = boot(d.path());
    a.on_core_failed();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert!(matches!(a.phase(), Phase::Shelf));
}
