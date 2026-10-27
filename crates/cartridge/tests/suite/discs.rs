//! A game on several discs: the emulator thread says how many there are and which is in, swaps
//! one in when asked, and hands the session the disc it swapped to so the game comes back on it.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::common;

use cartridge::audio::{AudioSink, StubSink};
use cartridge::emu::{CoreState, EmuHandle};
use cartridge::persist;
use cartridge_retro::{MockCore, RetroCore};
use cartridge_store::Platform;

fn ready(core: MockCore) -> EmuHandle {
    let emu = EmuHandle::spawn_for(
        Platform::Ps1,
        Box::new(core),
        PathBuf::from("mock.m3u"),
        StubSink::draining().ring(),
        None,
        None,
        None,
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while emu.state() == CoreState::Loading {
        assert!(Instant::now() < deadline, "the core never settled");
        std::thread::sleep(Duration::from_millis(2));
    }
    emu
}

fn eventually(what: &str, cond: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !cond() {
        assert!(Instant::now() < deadline, "{what} never happened");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn a_game_on_one_disc_says_so() {
    let emu = ready(MockCore::new());
    assert_eq!((emu.disc_count(), emu.disc_index()), (1, 0));
}

#[test]
fn a_disc_swap_round_trips_through_the_emulator_thread() {
    let emu = ready(MockCore::new().with_discs(3));
    assert_eq!(emu.disc_count(), 3);
    assert_eq!(emu.disc_index(), 0);

    emu.set_disc(2);
    eventually("the swap to disc 3", || emu.disc_index() == 2);
    assert_eq!(
        emu.take_swapped_disc(),
        Some((2, "mock disc 3".to_string())),
        "the session was not handed the disc swapped to"
    );
    assert_eq!(
        emu.take_swapped_disc(),
        None,
        "a swap was handed over twice"
    );

    emu.set_disc(7);
    emu.set_disc(0);
    eventually("the swap back to disc 1", || emu.disc_index() == 0);
    assert_eq!(
        emu.take_swapped_disc(),
        Some((0, "mock disc 1".to_string())),
        "a disc the game is not on was swapped in"
    );
}

#[test]
fn the_disc_a_game_was_swapped_to_is_kept_beside_its_save() {
    let d = common::tmp_root_with_carts(&[]);
    assert_eq!(persist::read_disc(d.path(), Platform::Ps1, "Game"), None);
    persist::write_disc(
        d.path(),
        Platform::Ps1,
        "Game",
        1,
        "/Games/PS1/Game (Disc 2).chd",
    )
    .expect("write the disc");
    assert_eq!(
        persist::read_disc(d.path(), Platform::Ps1, "Game"),
        Some((1, "/Games/PS1/Game (Disc 2).chd".to_string()))
    );
    assert!(d.path().join("Saves/PS1/Game.disc").exists());
}

#[test]
fn a_core_asked_to_start_on_a_disc_checks_its_file() {
    let mut core = MockCore::new().with_discs(2);
    core.set_initial_disc(1, "somewhere else");
    assert_eq!(core.disc_index(), 0, "a disc whose file moved was put in");
    core.set_initial_disc(1, "mock disc 2");
    assert_eq!(core.disc_index(), 1);
}
