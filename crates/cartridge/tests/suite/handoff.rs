use crate::common;

use cartridge::handoff;
use cartridge_input::{Action, Btn};
use cartridge_store::Platform;

/// A card with a PSP game and a Game Boy Advance one.
fn root() -> tempfile::TempDir {
    let d = common::tmp_root_with_carts(&["Emerald"]);
    let games = d.path().join("Games/PSP");
    std::fs::create_dir_all(&games).unwrap();
    std::fs::write(games.join("Racing Game.cso"), vec![0u8; 64]).unwrap();
    d
}

fn seat(s: &mut cartridge::session::Session, platform: Platform) {
    for _ in 0..32 {
        if s.app().selected_cart().map(|c| c.platform) == Some(platform) {
            break;
        }
        s.app_mut().apply(Action::GbaDown(Btn::R1));
        for _ in 0..60 {
            s.update(1.0 / 60.0);
        }
    }
    assert_eq!(
        s.app().selected_cart().map(|c| c.platform),
        Some(platform),
        "the shelf never came to {platform:?}"
    );
    s.app_mut().apply(Action::Insert);
}

#[test]
fn a_seated_psp_game_is_handed_over_once_its_cart_is_in() {
    let d = root();
    let (mut s, _motor) = common::session_with_platform(d.path());
    seat(&mut s, Platform::Psp);
    assert_eq!(s.handoff(), None, "handed over before the cart was in");
    for _ in 0..240 {
        s.update(1.0 / 60.0);
    }
    assert_eq!(
        s.handoff(),
        Some(d.path().join("Games/PSP/Racing Game.cso").as_path())
    );
}

#[test]
fn a_game_a_core_runs_is_never_handed_over() {
    let d = root();
    let (mut s, _motor) = common::session_with_platform(d.path());
    seat(&mut s, Platform::Gba);
    for _ in 0..240 {
        s.update(1.0 / 60.0);
    }
    assert_eq!(s.handoff(), None);
}

#[test]
fn the_request_names_the_program_and_the_game_on_two_lines_and_leaves_no_half_file() {
    let d = tempfile::tempdir().unwrap();
    let file = d.path().join("cartridge-handoff");
    handoff::request(
        &file,
        "psp",
        std::path::Path::new("/mnt/sdcard/Games/PSP/A B.cso"),
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "psp\n/mnt/sdcard/Games/PSP/A B.cso\n"
    );
    assert!(!d.path().join("cartridge-handoff.tmp").exists());
}

#[test]
fn a_handed_over_game_is_not_left_in_the_slot_to_be_run_again() {
    let d = root();
    let (mut s, _motor) = common::session_with_platform(d.path());
    seat(&mut s, Platform::Psp);
    for _ in 0..240 {
        s.update(1.0 / 60.0);
    }
    assert!(s.handoff().is_some());
    assert_eq!(
        cartridge_store::read_app_state(d.path()).cart.as_deref(),
        Some("Racing Game"),
        "the cart was not in the slot to begin with"
    );
    s.app_mut().forget_seated();
    let state = cartridge_store::read_app_state(d.path());
    assert_eq!((state.cart, state.cart_platform), (None, None));
}
