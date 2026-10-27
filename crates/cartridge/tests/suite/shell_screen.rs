use cartridge::shell_screen::{ShellScreen, PIGMENTS};
use cartridge_input::{Action, Btn};
use cartridge_store::{Outline, ShellChoice, ShellFinish};

#[test]
fn a_cart_in_its_own_colour_opens_on_original_and_solid_and_keeps_nothing() {
    let s = ShellScreen::of(None);
    assert_eq!((s.colour, s.finish), (0, ShellFinish::Solid));
    assert_eq!(s.choice(), None);
}

#[test]
fn right_on_the_colour_row_picks_the_next_pigment_and_down_reaches_the_finish() {
    let mut s = ShellScreen::of(None);
    assert!(s.input(Action::GbaDown(Btn::Right)));
    assert_eq!(s.choice().map(|c| c.colour), PIGMENTS[1].1);
    assert!(!s.input(Action::GbaDown(Btn::Down)));
    assert!(s.input(Action::GbaDown(Btn::Right)));
    assert_eq!(s.choice().map(|c| c.finish), Some(ShellFinish::Clear));
    assert!(s.input(Action::GbaDown(Btn::Left)));
    assert!(s.input(Action::GbaDown(Btn::Left)));
    assert_eq!(
        s.choice().map(|c| c.finish),
        Some(ShellFinish::Glitter),
        "did not wrap"
    );
}

#[test]
fn a_kept_pigment_and_its_outline_come_back_as_they_were() {
    let kept = ShellChoice {
        outline: Outline::Rounded,
        colour: PIGMENTS[4].1.unwrap(),
        finish: ShellFinish::Glitter,
    };
    let s = ShellScreen::of(Some(kept));
    assert_eq!(s.colour, 4);
    assert_eq!(s.choice(), Some(kept));
}
