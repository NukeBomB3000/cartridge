use cartridge::core::ghosting_option;
use cartridge::video_mode::{screen_on, VideoMode};
use cartridge_gfx::Canvas;
use cartridge_retro::{LibretroCore, RetroCore};
use cartridge_store::Core;
use cartridge_store::Platform;

use crate::common;

fn dylib() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor")
        .join(cartridge::core::dylib_name(Core::GenesisPlusGx))
}

/// A blank cart image, all NOPs, under `ext`: Genesis Plus GX tells a Game Gear game from a
/// Master System one by its file's extension.
fn blank_cart(ext: &str) -> tempfile::NamedTempFile {
    let f = tempfile::Builder::new()
        .suffix(&format!(".{ext}"))
        .tempfile()
        .unwrap();
    std::fs::write(f.path(), vec![0u8; 32 * 1024]).unwrap();
    f
}

/// The size of the picture Genesis Plus GX draws for a blank `ext` cart, centred on black in the
/// GBA's screen when it is smaller.
fn drawn(ext: &str) -> Option<(u32, u32)> {
    let path = dylib();
    if !path.exists() {
        eprintln!("no genesis_plus_gx dylib on this host, skipping");
        return None;
    }
    let mut core = LibretroCore::open(&path).expect("open core");
    let rom = blank_cart(ext);
    RetroCore::load(&mut core, rom.path()).expect("load the cart");
    for _ in 0..3 {
        core.run_frame(Default::default());
    }
    Some(core.video_size())
}

#[test]
fn a_game_gear_game_draws_its_160x144_screen_inside_the_gbas_as_a_game_boy_game_does() {
    let _g = common::core_lock();
    if let Some(size) = drawn("gg") {
        assert_eq!(
            size,
            (240, 160),
            "not centred in the GBA's screen as a Game Boy game is"
        );
    }
}

#[test]
fn a_master_system_game_draws_its_256x192_screen() {
    let _g = common::core_lock();
    if let Some(size) = drawn("sms") {
        assert_eq!(size, (256, 192));
    }
}

#[test]
fn a_game_gear_game_ghosts_through_genesis_plus_gxs_lcd_filter() {
    assert_eq!(
        ghosting_option(Core::GenesisPlusGx, true),
        Some(("genesis_plus_gx_lcd_filter", "enabled"))
    );
    assert_eq!(
        ghosting_option(Core::GenesisPlusGx, false),
        Some(("genesis_plus_gx_lcd_filter", "disabled"))
    );
}

#[test]
fn a_game_gear_game_at_its_actual_size_stands_as_a_game_boy_game_at_3x_over_the_body() {
    for (c, x) in [(Canvas::FOUR_THREE, 80.0), (Canvas::WIDE, 120.0)] {
        let s = screen_on(c, Platform::Gg, VideoMode::Actual);
        assert_eq!(s.view.rect, [x, 0.0, 480.0, 432.0], "{c:?}");
        assert_eq!(
            s.view.rect,
            screen_on(c, Platform::Gb, VideoMode::Actual).view.rect
        );
    }
}

#[test]
fn a_master_system_game_at_its_actual_size_is_its_lines_at_twice() {
    let s = screen_on(Canvas::FOUR_THREE, Platform::Sms, VideoMode::Actual);
    assert_eq!(s.view.rect, [64.0, 48.0, 512.0, 384.0]);
    let s = screen_on(Canvas::WIDE, Platform::Sms, VideoMode::Actual);
    assert_eq!(s.view.rect, [104.0, 48.0, 512.0, 384.0]);
}
