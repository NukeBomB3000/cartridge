//! A core's pictures reach the screen at their own size, whatever it is and however often it
//! changes, as a PlayStation game's does between its menus and its game.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use cartridge::audio::{AudioSink, StubSink};
use cartridge::emu::{CoreState, EmuHandle, Speed};
use cartridge::persist::Snapshot;
use cartridge_retro::{MockCore, RetroCore};
use cartridge_store::Platform;

fn running(platform: Platform, core: MockCore) -> EmuHandle {
    let emu = EmuHandle::spawn_for(
        platform,
        Box::new(core),
        PathBuf::from("mock"),
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
    emu.set_speed(Speed::Normal);
    emu
}

/// The first picture that `until` accepts, waiting for the thread to publish it.
fn picture(emu: &EmuHandle, until: impl Fn((u32, u32)) -> bool) -> ((u32, u32), Vec<u8>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(f) = emu.latest_frame() {
            if until(f.size()) {
                return (f.size(), f.to_vec());
            }
        }
        assert!(Instant::now() < deadline, "no such picture was published");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn a_snes_picture_reaches_the_screen_at_its_own_size() {
    let emu = running(Platform::Snes, MockCore::sized(256, 224));
    let (size, px) = picture(&emu, |_| true);
    assert_eq!(size, (256, 224));
    assert_eq!(px.len(), 256 * 224 * 4);
}

#[test]
fn a_picture_that_changes_size_as_the_game_runs_is_followed() {
    let emu = running(
        Platform::Ps1,
        MockCore::sized(320, 240).resized_at(20, 640, 480),
    );
    let (before, _) = picture(&emu, |_| true);
    assert_eq!(before, (320, 240), "the game's first picture");
    let (after, px) = picture(&emu, |s| s != (320, 240));
    assert_eq!(after, (640, 480));
    assert_eq!(
        px.len(),
        640 * 480 * 4,
        "the picture is not as big as it says"
    );
}

#[test]
fn a_game_boy_picture_still_fills_the_gba_screen() {
    let emu = running(Platform::Gb, MockCore::new());
    let (size, _) = picture(&emu, |_| true);
    assert_eq!(size, (240, 160));
}

#[test]
fn a_save_state_s_screenshot_is_the_picture_at_its_own_size() {
    let emu = running(Platform::Ps1, MockCore::sized(368, 240));
    picture(&emu, |_| true);
    let png = emu.snapshot().thumb().expect("no screenshot came back");
    let reader = png::Decoder::new(png.as_slice())
        .read_info()
        .expect("the screenshot is no png");
    let info = reader.info();
    assert_eq!((info.width, info.height), (368, 240));
}

#[test]
fn a_mock_draws_whatever_size_it_is_asked_for() {
    for (w, h) in [(256, 224), (320, 240), (512, 448), (640, 480)] {
        let core = MockCore::sized(w, h);
        assert_eq!(core.video_size(), (w, h));
        assert_eq!(core.video_xrgb8888().len(), (w * h * 4) as usize);
    }
}
