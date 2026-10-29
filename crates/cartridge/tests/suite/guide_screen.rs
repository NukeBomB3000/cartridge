//! The game guide a short MENU press opens over a game: found in Guides/<system> by the game's
//! name, held paused under it, paged, scrolled and sized, closed with B or MENU, and opened
//! again where it was left, even after the next boot.

use crate::common;
use crate::keys::root_with_games;

use cartridge::app::{App, Phase};
use cartridge::guide_screen::{find_guides, match_name, GuideMarks, Tab};
use cartridge_input::{Action, Btn};
use cartridge_store::{read_app_state, Platform};
use cartridge_ui::guide::GUIDE_SIZE_DEFAULT;
use tempfile::TempDir;

const GAME: &str = "Golden Sun - Die Vergessene Epoche";

fn card(with_guide: bool) -> TempDir {
    let d = root_with_games(Platform::Gba, &[GAME]);
    if with_guide {
        let dir = d.path().join("Guides/GBA");
        std::fs::create_dir_all(&dir).unwrap();
        let text: String = (0..400)
            .map(|i| {
                format!(
                    " Absatz {i}: ein Satz, der lang genug ist, um umbrochen zu werden, ja.\n\n"
                )
            })
            .collect();
        std::fs::write(dir.join(format!("{GAME}.txt")), text).unwrap();
    }
    d
}

fn playing(d: &TempDir) -> App {
    let mut a = common::app_playing_in(d.path(), GAME);
    a.set_platform(Platform::Gba);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
    a
}

#[test]
fn guides_are_found_by_the_game_s_name_whatever_its_brackets_and_case() {
    assert_eq!(match_name("Golden Sun (Germany) [b1]"), "goldensun");
    let d = TempDir::new().unwrap();
    std::fs::write(d.path().join("golden sun.TXT"), "x").unwrap();
    std::fs::create_dir(d.path().join("Golden Sun")).unwrap();
    std::fs::write(d.path().join("Golden Sun/b.html"), "x").unwrap();
    std::fs::write(d.path().join("Golden Sun/notes.png"), "x").unwrap();
    std::fs::write(d.path().join("Other.txt"), "x").unwrap();
    let found = find_guides(d.path(), "Golden Sun (Germany)");
    let names: Vec<_> = found
        .iter()
        .map(|p| {
            p.strip_prefix(d.path())
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    assert_eq!(names, vec!["Golden Sun/b.html", "golden sun.TXT"]);
}

#[test]
fn the_marks_read_back_what_they_wrote() {
    let mut m = GuideMarks::default();
    m.size = Some(4);
    m.at.insert("GBA/Golden Sun.txt".into(), 1234);
    assert_eq!(GuideMarks::parse(&m.print()), m);
}

#[test]
fn a_short_menu_press_opens_the_guide_and_holds_the_game() {
    let d = card(true);
    let mut a = playing(&d);
    a.apply(Action::QuickMenu);
    assert!(a.guide_open());
    assert!(a.game_menu_open(), "the game is not held under the guide");
    assert!(a.take_guide_page().is_some(), "the page was not drawn");
    assert!(
        a.take_guide_page().is_none(),
        "an unchanged page was drawn again"
    );
    a.apply(Action::GbaDown(Btn::B));
    assert!(!a.guide_open());
    assert!(!a.game_menu_open());
}

#[test]
fn a_game_with_no_guide_still_opens_the_menu_and_says_so() {
    let d = card(false);
    let mut a = playing(&d);
    a.apply(Action::QuickMenu);
    assert!(a.guide_open());
    assert_eq!(a.guide().unwrap().tab, Tab::Guide);
    assert!(a.take_guide_page().is_some());
    assert_eq!(a.toast(), None);
    // and says what the file is to be called: the game's name as the card has it
    let hint = a.guide().unwrap().text_of_tab();
    assert!(
        hint.contains(&format!("Guides/GBA/{GAME}.txt")),
        "the hint does not name the file: {hint}"
    );
}

#[test]
fn l1_and_r1_turn_the_tabs_round_and_left_and_right_page() {
    let d = card(true);
    let mut a = playing(&d);
    a.apply(Action::QuickMenu);
    let tab = |a: &App| a.guide().unwrap().tab;
    assert_eq!(tab(&a), Tab::Guide);
    a.apply(Action::GbaDown(Btn::R1));
    assert_eq!(tab(&a), Tab::Achievements);
    a.apply(Action::GbaDown(Btn::R1));
    assert_eq!(tab(&a), Tab::Settings);
    a.apply(Action::GbaDown(Btn::R1));
    assert_eq!(tab(&a), Tab::Guide, "round from the last tab");
    a.apply(Action::GbaDown(Btn::L1));
    assert_eq!(tab(&a), Tab::Settings);
    a.apply(Action::GbaDown(Btn::L1));
    a.apply(Action::GbaDown(Btn::L1));
    assert_eq!(tab(&a), Tab::Guide);
    let top = a.guide().unwrap().top;
    a.apply(Action::GbaDown(Btn::Right));
    assert!(a.guide().unwrap().top > top, "Right pages the guide");
    a.apply(Action::GbaDown(Btn::Left));
    assert_eq!(a.guide().unwrap().top, top, "Left pages back");
}

#[test]
fn the_guide_keeps_its_place_while_another_tab_shows() {
    let d = card(true);
    let mut a = playing(&d);
    a.apply(Action::QuickMenu);
    a.apply(Action::GbaDown(Btn::Right));
    a.apply(Action::GbaDown(Btn::Right));
    let top = a.guide().unwrap().top;
    assert!(top > 0.0);
    a.apply(Action::GbaDown(Btn::R1));
    a.apply(Action::GbaDown(Btn::L1));
    assert_eq!(a.guide().unwrap().top, top);
}

#[test]
fn the_settings_tab_steps_a_row_with_left_and_right_and_keeps_it() {
    let d = card(false);
    let mut a = playing(&d);
    a.apply(Action::QuickMenu);
    a.apply(Action::GbaDown(Btn::L1));
    assert_eq!(a.guide().unwrap().tab, Tab::Settings);
    let before = read_app_state(d.path()).brightness;
    a.apply(Action::GbaDown(Btn::Left));
    assert_eq!(
        read_app_state(d.path()).brightness,
        before - 1,
        "Left steps back"
    );
    a.apply(Action::GbaDown(Btn::Right));
    a.apply(Action::GbaDown(Btn::Right));
    assert_eq!(
        read_app_state(d.path()).brightness,
        before + 1,
        "Right steps on"
    );
    // Down moves the cursor to blue light
    a.apply(Action::GbaDown(Btn::Down));
    a.apply(Action::GbaDown(Btn::Right));
    assert_eq!(read_app_state(d.path()).blue_light, 1);
    // and the core's options are asked for
    assert!(a.wants_guide_options());
}

#[test]
fn pages_lines_and_sizes_move_the_guide_and_it_opens_again_where_it_was_left() {
    let d = card(true);
    let mut a = playing(&d);
    a.apply(Action::QuickMenu);
    let top = |a: &App| a.guide().unwrap().top;
    a.apply(Action::GbaDown(Btn::Right));
    let paged = top(&a);
    assert!(paged > 0.0);
    a.apply(Action::GbaDown(Btn::Down));
    assert!(top(&a) > paged);
    a.apply(Action::GbaDown(Btn::Up));
    assert_eq!(top(&a), paged);
    a.apply(Action::GbaDown(Btn::Left));
    assert_eq!(top(&a), 0.0);
    for _ in 0..5 {
        a.apply(Action::GbaDown(Btn::Right));
    }
    let at = a.guide().unwrap().at();
    a.apply(Action::GbaDown(Btn::R2));
    assert_eq!(a.guide().unwrap().size, GUIDE_SIZE_DEFAULT + 1);
    let after = a.guide().unwrap().at();
    assert!(
        after <= at && at - after < 200,
        "the size lost the place: {at} → {after}"
    );
    a.apply(Action::QuickMenu);
    assert!(!a.guide_open(), "MENU did not close the guide");

    // the next boot opens it at the same place and size
    let mut b = playing(&d);
    b.apply(Action::QuickMenu);
    assert_eq!(b.guide().unwrap().size, GUIDE_SIZE_DEFAULT + 1);
    assert_eq!(b.guide().unwrap().at(), after);
}

#[test]
fn a_second_menu_press_still_opens_the_save_states() {
    let d = card(true);
    let mut a = playing(&d);
    a.apply(Action::QuickMenu);
    a.apply(Action::Polaroids);
    assert!(!a.guide_open(), "the double press left the guide open");
    // with no states kept the save states refuse, as they do with no guide in the way
    assert!(matches!(
        a.phase(),
        Phase::Playing { .. } | Phase::Polaroids { .. }
    ));
}

#[test]
fn a_handheld_s_rewind_and_fast_forward_size_the_text_over_the_guide() {
    let d = card(true);
    let mut a = playing(&d);
    a.apply(Action::QuickMenu);
    a.apply(Action::FfStart);
    a.apply(Action::FfStop);
    assert_eq!(a.guide().unwrap().size, GUIDE_SIZE_DEFAULT + 1);
    a.apply(Action::RewindStart);
    a.apply(Action::RewindStop);
    a.apply(Action::RewindStart);
    assert_eq!(a.guide().unwrap().size, GUIDE_SIZE_DEFAULT - 1);
}
