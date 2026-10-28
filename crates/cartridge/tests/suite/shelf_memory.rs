use crate::common;

use cartridge::app::App;
use cartridge_input::{Action, Btn};
use cartridge_store::{read_app_state, write_app_state, AppState, Platform};
use common::{boot, tmp_root_with_carts};

const GAMES: [&str; 5] = ["Advance Wars", "Emerald", "Fire Emblem", "Metroid", "Zelda"];

fn step_right(a: &mut App) {
    a.apply(Action::ShelfRight);
    a.apply(Action::GbaUp(Btn::Right));
    for _ in 0..60 {
        a.update(1.0 / 60.0);
    }
}

fn settle(a: &mut App, seconds: u32) {
    for _ in 0..seconds * 60 {
        a.update(1.0 / 60.0);
    }
}

fn remembering(stem: &str, index: usize) -> AppState {
    AppState {
        clock_set: true,
        browsed: Some(stem.into()),
        browsed_platform: Some(Platform::Gba),
        browsed_index: index,
        ..Default::default()
    }
}

#[test]
fn the_shelf_opens_on_the_cart_it_was_left_on() {
    let d = tmp_root_with_carts(&GAMES);
    write_app_state(d.path(), &remembering("Metroid", 3)).unwrap();
    let a = boot(d.path());
    assert_eq!(a.selected_stem(), Some("Metroid"));
}

#[test]
fn the_cart_browsed_to_is_written_once_the_shelf_rests() {
    let d = tmp_root_with_carts(&GAMES);
    let mut a = boot(d.path());
    step_right(&mut a);
    step_right(&mut a);
    settle(&mut a, 3);
    let s = read_app_state(d.path());
    assert_eq!(s.browsed.as_deref(), Some("Fire Emblem"));
    assert_eq!(s.browsed_platform, Some(Platform::Gba));
    assert_eq!(s.browsed_index, 2);
    let again = boot(d.path());
    assert_eq!(again.selected_stem(), Some("Fire Emblem"));
}

#[test]
fn a_removed_cart_leaves_the_shelf_at_its_place() {
    let d = tmp_root_with_carts(&GAMES);
    write_app_state(d.path(), &remembering("Deleted", 3)).unwrap();
    let a = boot(d.path());
    assert_eq!(
        a.selected_stem(),
        Some("Metroid"),
        "the same place on the shelf"
    );
}

#[test]
fn a_shelf_that_got_shorter_opens_on_its_last_cart() {
    let d = tmp_root_with_carts(&["Advance Wars", "Emerald"]);
    write_app_state(d.path(), &remembering("Deleted", 40)).unwrap();
    let a = boot(d.path());
    assert_eq!(a.selected_stem(), Some("Emerald"));
}

#[test]
fn a_system_with_no_games_left_opens_the_shelf_as_usual() {
    let d = tmp_root_with_carts(&GAMES);
    let mut s = remembering("Sonic", 2);
    s.browsed_platform = Some(Platform::Md);
    write_app_state(d.path(), &s).unwrap();
    let a = boot(d.path());
    assert_eq!(a.selected_stem(), Some("Advance Wars"));
}

#[test]
fn a_state_file_from_before_the_memory_still_opens() {
    let d = tmp_root_with_carts(&GAMES);
    write_app_state(
        d.path(),
        &AppState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let text = std::fs::read_to_string(d.path().join("Config/cartridge.state")).unwrap();
    let old: String = text
        .lines()
        .filter(|l| !l.starts_with("browsed"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(d.path().join("Config/cartridge.state"), old).unwrap();
    let a = boot(d.path());
    assert_eq!(a.selected_stem(), Some("Advance Wars"));
}
