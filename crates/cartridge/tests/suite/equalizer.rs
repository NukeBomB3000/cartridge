use cartridge::audio::{Equalizer, EQ_HZ};
use cartridge::equalizer_screen::{preset_of, EqualizerScreen, Outcome, PRESETS};
use cartridge_input::{Action, Btn};

/// How loud a sine at `hz` comes out of the equalizer at `gains`, against how loud it went in.
fn gain_at(gains: [i8; 5], hz: f32) -> f32 {
    let mut eq = Equalizer::default();
    eq.set(gains, 48_000);
    let samples: Vec<i16> = (0..48_000 * 2)
        .map(|i| {
            let t = (i / 2) as f32 / 48_000.0;
            ((std::f32::consts::TAU * hz * t).sin() * 8_000.0) as i16
        })
        .collect();
    let mut out = samples.clone();
    eq.process(&mut out);
    let peak = |s: &[i16]| s[48_000..].iter().map(|v| v.unsigned_abs()).max().unwrap() as f32;
    peak(&out) / peak(&samples)
}

#[test]
fn a_band_raised_6_db_doubles_its_middle_and_leaves_the_others_near_alone() {
    let mut gains = [0; 5];
    gains[2] = 6;
    let at = gain_at(gains, EQ_HZ[2]);
    assert!((1.9..=2.1).contains(&at), "1 kHz came out at {at}x");
    let far = gain_at(gains, EQ_HZ[0]);
    assert!((0.95..=1.2).contains(&far), "150 Hz came out at {far}x");
}

#[test]
fn flat_passes_the_sound_untouched() {
    let mut eq = Equalizer::default();
    eq.set([0; 5], 48_000);
    let mut s = vec![1234i16, -5678, 32_000, -32_000];
    eq.process(&mut s);
    assert_eq!(s, [1234, -5678, 32_000, -32_000]);
}

#[test]
fn a_band_turned_down_quietens_it_and_a_loud_one_clips_rather_than_wraps() {
    let mut gains = [0; 5];
    gains[4] = -12;
    assert!(gain_at(gains, 12_000.0) < 0.35);
    let mut eq = Equalizer::default();
    eq.set([12; 5], 48_000);
    let mut s: Vec<i16> = (0..4_800)
        .map(|i| if i % 40 < 20 { 30_000 } else { -30_000 })
        .collect();
    eq.process(&mut s);
    assert!(
        s.iter().any(|&v| v == i16::MAX || v == i16::MIN),
        "never clipped"
    );
}

#[test]
fn the_screen_moves_between_bands_and_steps_a_db_at_a_time_within_reach() {
    let mut screen = EqualizerScreen::default();
    let press = |s: &mut EqualizerScreen, g, b| s.input(g, Action::GbaDown(b));
    assert_eq!(
        press(&mut screen, [0; 5], Btn::Up),
        Outcome::Changed([1, 0, 0, 0, 0])
    );
    press(&mut screen, [0; 5], Btn::Right);
    press(&mut screen, [0; 5], Btn::Right);
    assert_eq!(
        press(&mut screen, [0; 5], Btn::Down),
        Outcome::Changed([0, 0, -1, 0, 0])
    );
    assert_eq!(press(&mut screen, [0, 0, 12, 0, 0], Btn::Up), Outcome::Stay);
    for _ in 0..9 {
        press(&mut screen, [0; 5], Btn::Right);
    }
    assert_eq!(screen.band, 4);
    press(&mut screen, [0; 5], Btn::A);
    assert!(screen.playing);
    assert_eq!(press(&mut screen, [0; 5], Btn::B), Outcome::Close);
    assert!(!screen.playing, "the test sound outlived the screen");
}

#[test]
fn y_steps_through_the_presets_and_a_hand_made_setting_starts_them_over() {
    let mut screen = EqualizerScreen::default();
    let mut gains = [0; 5];
    for (name, want) in PRESETS.iter().skip(1).chain(PRESETS.iter().take(1)) {
        let Outcome::Changed(next) = screen.input(gains, Action::GbaDown(Btn::Y)) else {
            panic!("Y changed nothing from {gains:?}");
        };
        assert_eq!((preset_of(next), next), (Some(*name), *want));
        gains = next;
    }
    assert_eq!(
        screen.input([5, 0, 0, 0, 0], Action::GbaDown(Btn::Y)),
        Outcome::Changed(PRESETS[0].1)
    );
}
