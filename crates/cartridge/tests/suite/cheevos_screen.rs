use cartridge::cheevos::Status;
use cartridge::cheevos_screen::{forget_account, keep_account, CheevosScreen, Outcome, Sel};
use cartridge_cheevos::Credentials;
use cartridge_input::{Action, Btn};

use crate::common::tmp_root_with_carts;

fn press(s: &mut CheevosScreen, btn: Btn, typed: &mut Option<(String, String)>) -> Outcome {
    s.input(Action::GbaDown(btn), "", typed)
}

fn type_in(s: &mut CheevosScreen, text: &str) {
    let t = s.typing.as_mut().expect("the keyboard is up");
    for c in text.chars() {
        t.keys.type_char(c);
    }
}

#[test]
fn a_toggles_the_service_and_b_leaves() {
    let mut s = CheevosScreen::default();
    let mut typed = None;
    assert_eq!(press(&mut s, Btn::A, &mut typed), Outcome::Toggle);
    assert_eq!(press(&mut s, Btn::Right, &mut typed), Outcome::Toggle);
    assert_eq!(press(&mut s, Btn::B, &mut typed), Outcome::Close);
}

#[test]
fn an_account_is_typed_as_a_user_name_then_a_password() {
    let mut s = CheevosScreen::default();
    let mut typed = None;
    press(&mut s, Btn::Down, &mut typed);
    assert_eq!(s.sel, Sel::Account);
    press(&mut s, Btn::A, &mut typed);
    assert_eq!(
        s.typing.as_ref().unwrap().prompt(),
        "RetroAchievements user name"
    );
    type_in(&mut s, "player");
    assert_eq!(press(&mut s, Btn::Start, &mut typed), Outcome::Stay);
    assert!(s.typing.as_ref().unwrap().prompt().contains("player"));
    type_in(&mut s, "secret");
    assert_eq!(press(&mut s, Btn::Start, &mut typed), Outcome::SignIn);
    assert_eq!(typed, Some(("player".to_string(), "secret".to_string())));
    assert!(s.typing.is_none());
}

#[test]
fn b_on_an_empty_keyboard_gives_the_typing_up() {
    let mut s = CheevosScreen::default();
    let mut typed = None;
    press(&mut s, Btn::Down, &mut typed);
    press(&mut s, Btn::A, &mut typed);
    press(&mut s, Btn::B, &mut typed);
    assert!(s.typing.is_none());
    assert_eq!(typed, None);
}

#[test]
fn y_signs_the_account_out_only_on_the_account_row() {
    let mut s = CheevosScreen::default();
    let mut typed = None;
    assert_eq!(press(&mut s, Btn::Y, &mut typed), Outcome::Stay);
    press(&mut s, Btn::Down, &mut typed);
    assert_eq!(press(&mut s, Btn::Y, &mut typed), Outcome::SignOut);
}

#[test]
fn the_notes_follow_the_sign_in() {
    let lines = |status, online, on| CheevosScreen::status_lines(&status, online, on);
    assert!(lines(Status::SignedOut, true, true)[1].contains("Softcore"));
    assert!(lines(Status::SigningIn, false, true)[0].contains("Wi-Fi"));
    assert_eq!(lines(Status::SigningIn, true, true), ["Signing in..."]);
    assert!(lines(Status::SignedIn("me".into()), true, false)[0].contains("off"));
    assert_eq!(
        lines(Status::Rejected("bad password".into()), true, true),
        ["Sign-in refused:", "bad password"]
    );
}

#[test]
fn the_account_is_kept_for_the_client_and_forgotten_again() {
    let d = tmp_root_with_carts(&[]);
    keep_account(d.path(), "player", "secret").unwrap();
    let c = Credentials::read(d.path()).expect("kept");
    assert_eq!(c.username, "player");
    assert!(matches!(c.secret, cartridge_cheevos::Secret::Password(p) if p == "secret"));
    // a new account replaces the old one, a token kept for the old with it
    cartridge_store::ini::write(d.path(), cartridge_cheevos::CHEEVOS_FILE, "token", "old").unwrap();
    keep_account(d.path(), "other", "pw").unwrap();
    let c = Credentials::read(d.path()).unwrap();
    assert_eq!(c.username, "other");
    assert!(matches!(c.secret, cartridge_cheevos::Secret::Password(_)));
    forget_account(d.path());
    assert!(Credentials::read(d.path()).is_none());
}
