use crate::common;

use std::time::{Duration, Instant};

use common::{session_with_platform, tmp_root_with_carts, tmp_root_with_real_carts};
use omega_input::{Btn, Millis, RawEvent, MENU_HOLD_MS, POWER_HOLD_MS};
use omega_store::{write_app_state, AppState};
use omega_zero::app::Phase;
use omega_zero::session::Session;

const FRAME_MS: Millis = 16;
const DT: f32 = 1.0 / 60.0;

const STRONG: u32 = 0;

#[test]
fn what_the_core_asks_for_reaches_the_motor() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    s.core_rumble()
        .expect("a seated cart has a core")
        .set(0, STRONG, u16::MAX);
    step(&mut s, &mut now);
    assert_eq!(motor.last(), u16::MAX, "the core asked and nothing moved");

    let pressed = now;
    event(&mut s, RawEvent::Down(Btn::Menu), &mut now);
    while now < pressed + MENU_HOLD_MS + FRAME_MS {
        step(&mut s, &mut now);
    }
    assert_eq!(motor.last(), 0, "the motor outlived the cart");
    step(&mut s, &mut now);
    assert_eq!(motor.last(), 0, "the next frame turned it back on");
}

#[test]
fn with_rumble_off_the_motor_stays_still_whatever_the_core_asks() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    write_app_state(
        d.path(),
        &AppState {
            rumble: false,
            ..AppState::default()
        },
    )
    .expect("write omega-zero.state");
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    let core = s.core_rumble().expect("a seated cart has a core");
    core.set(0, STRONG, u16::MAX);
    assert_eq!(
        core.strength(),
        u16::MAX,
        "the core is not asking, so this test proves nothing"
    );
    for _ in 0..5 {
        step(&mut s, &mut now);
    }
    assert_eq!(motor.last(), 0, "the motor moved with rumble off");
}

#[test]
fn ejecting_stops_the_motor() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    s.rumble(u16::MAX);
    assert_ne!(motor.last(), 0);
    s.feed([RawEvent::Down(Btn::Menu)], 0);
    s.feed([], MENU_HOLD_MS + 1);
    assert_eq!(
        motor.last(),
        0,
        "the motor kept running after the cart came out"
    );
}

#[test]
fn dozing_stops_the_motor() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    s.rumble(u16::MAX);
    s.feed([RawEvent::Down(Btn::Lid)], 0);
    assert_eq!(motor.last(), 0);
}

#[test]
fn the_power_button_stops_the_motor() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    s.rumble(u16::MAX);
    s.feed([RawEvent::Down(Btn::Power)], 0);
    assert_eq!(motor.last(), 0);
}

fn step(s: &mut Session, now: &mut Millis) {
    *now += FRAME_MS;
    s.feed([], *now);
    s.update(DT);
}

fn event(s: &mut Session, ev: RawEvent, now: &mut Millis) {
    *now += FRAME_MS;
    s.feed([ev], *now);
    s.update(DT);
}

fn play(s: &mut Session, now: &mut Millis) {
    event(s, RawEvent::Down(Btn::A), now);
    event(s, RawEvent::Up(Btn::A), now);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline, "the cart never seated");
        step(s, now);
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn the_power_menu_takes_the_motor_down() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    s.core_rumble()
        .expect("a seated cart has a core")
        .set(0, STRONG, u16::MAX);
    step(&mut s, &mut now);
    assert_eq!(
        motor.last(),
        u16::MAX,
        "the motor should be running, or this test proves nothing"
    );

    let pressed = now;
    event(&mut s, RawEvent::Down(Btn::Power), &mut now);
    while now < pressed + POWER_HOLD_MS + FRAME_MS {
        step(&mut s, &mut now);
    }
    assert!(s.app().powering_off(), "the hold did not power off");
    step(&mut s, &mut now);
    assert_eq!(motor.last(), u16::MAX, "powering off did not thump");
    let going = now;
    while now < going + omega_zero::app::THUMP_MS + FRAME_MS {
        step(&mut s, &mut now);
    }
    assert_eq!(motor.last(), 0, "the cart kept buzzing as it went down");
}

#[test]
fn a_short_tap_runs_the_motor_long_enough_to_feel() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    let core = s.core_rumble().expect("a seated cart has a core").clone();
    core.set(0, STRONG, 20_000);
    step(&mut s, &mut now);
    let start = s.app().now();
    assert_eq!(motor.last(), u16::MAX, "a weak ask buzzes at the strength Settings has");

    core.set(0, STRONG, 0);
    while s.app().now() + 2 * FRAME_MS < start + omega_zero::session::RUMBLE_MIN_MS {
        step(&mut s, &mut now);
        let held = s.app().now() - start;
        assert_eq!(
            motor.last(),
            u16::MAX,
            "a tap stopped the motor after {held} ms"
        );
    }
    while s.app().now() < start + omega_zero::session::RUMBLE_MIN_MS + FRAME_MS {
        step(&mut s, &mut now);
    }
    assert_eq!(
        motor.last(),
        0,
        "the motor kept running after the game stopped asking"
    );
}

#[test]
fn a_buzz_is_cut_off_at_the_longest_the_motor_runs() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    let core = s.core_rumble().expect("a seated cart has a core").clone();
    core.set(0, STRONG, u16::MAX);
    step(&mut s, &mut now);
    let start = s.app().now();
    while s.app().now() + FRAME_MS < start + omega_zero::session::RUMBLE_MAX_MS {
        step(&mut s, &mut now);
        assert_eq!(motor.last(), u16::MAX, "the buzz was cut short");
    }
    while s.app().now() < start + 2 * omega_zero::session::RUMBLE_MAX_MS {
        step(&mut s, &mut now);
        assert_eq!(motor.last(), 0, "a long pulse droned on");
    }

    core.set(0, STRONG, 0);
    step(&mut s, &mut now);
    core.set(0, STRONG, u16::MAX);
    step(&mut s, &mut now);
    assert_eq!(motor.last(), u16::MAX, "the next hit never buzzed");
}

#[test]
fn the_shelf_ticks_the_motor_as_it_moves_on() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    step(&mut s, &mut now);
    step(&mut s, &mut now);
    assert_eq!(motor.last(), 0, "the shelf buzzed before it moved");
    event(&mut s, RawEvent::Down(Btn::Right), &mut now);
    event(&mut s, RawEvent::Up(Btn::Right), &mut now);
    let mut felt = motor.last() > 0;
    step(&mut s, &mut now);
    felt |= motor.last() > 0;
    assert!(felt, "the shelf moved on without a tick");
    let moved = now;
    while now < moved + omega_zero::app::SHELF_BUZZ_MS + 3 * FRAME_MS {
        step(&mut s, &mut now);
    }
    assert_eq!(motor.last(), 0, "the tick ran on");
}

#[test]
fn a_cart_seating_thumps_the_motor() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    event(&mut s, RawEvent::Down(Btn::A), &mut now);
    event(&mut s, RawEvent::Up(Btn::A), &mut now);
    let mut felt = false;
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline, "the cart never seated");
        step(&mut s, &mut now);
        felt |= motor.last() == u16::MAX;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(felt, "the cart seated without a thump");
}

#[test]
fn taking_a_cart_out_thumps_the_motor() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    play(&mut s, &mut now);
    let seated = now;
    while now < seated + omega_zero::app::THUMP_MS + FRAME_MS {
        step(&mut s, &mut now);
    }
    assert_eq!(motor.last(), 0, "the seating thump ran on into the game");
    let pressed = now;
    event(&mut s, RawEvent::Down(Btn::Menu), &mut now);
    while now < pressed + MENU_HOLD_MS + FRAME_MS {
        step(&mut s, &mut now);
    }
    event(&mut s, RawEvent::Up(Btn::Menu), &mut now);
    let mut felt = false;
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Shelf) {
        assert!(Instant::now() < deadline, "the cart never came out");
        step(&mut s, &mut now);
        felt |= motor.last() == u16::MAX;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(felt, "the cart came out without a thump");
}

#[test]
fn with_rumble_off_the_launcher_stays_still() {
    let d = tmp_root_with_real_carts(&["Advance Wars", "Emerald"]);
    write_app_state(
        d.path(),
        &AppState {
            rumble: false,
            ..AppState::default()
        },
    )
    .unwrap();
    let (mut s, motor) = session_with_platform(d.path());
    let mut now = 0;
    step(&mut s, &mut now);
    event(&mut s, RawEvent::Down(Btn::Right), &mut now);
    event(&mut s, RawEvent::Up(Btn::Right), &mut now);
    for _ in 0..5 {
        step(&mut s, &mut now);
        assert_eq!(motor.last(), 0, "the shelf ticked with rumble off");
    }
}
