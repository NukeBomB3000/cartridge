use cartridge_gfx::{blit_rect, out_h, out_w, panel_rect, Rotation};

/// Where viewport point `p` (0..1, y up) samples the upright picture, as the turned blit
/// computes it.
fn sample(rot: Rotation, p: (f32, f32)) -> (f32, f32) {
    let m = rot.uv_turn();
    let (s, t) = (p.0 - 0.5, p.1 - 0.5);
    (m[0] * s + m[2] * t + 0.5, m[1] * s + m[3] * t + 0.5)
}

#[test]
fn a_turn_is_read_to_the_nearest_quarter() {
    assert_eq!(Rotation::from_ccw_degrees(0), Rotation::R0);
    assert_eq!(Rotation::from_ccw_degrees(90), Rotation::R90);
    assert_eq!(Rotation::from_ccw_degrees(180), Rotation::R180);
    assert_eq!(Rotation::from_ccw_degrees(270), Rotation::R270);
    assert_eq!(Rotation::from_ccw_degrees(360), Rotation::R0);
    assert_eq!(Rotation::from_ccw_degrees(88), Rotation::R90);
}

#[test]
fn a_quarter_turn_stands_a_portrait_panel_on_its_side() {
    assert_eq!(Rotation::R90.upright((480, 640)), (640, 480));
    assert_eq!(Rotation::R270.upright((480, 640)), (640, 480));
    assert_eq!(Rotation::R180.upright((640, 480)), (640, 480));
    assert_eq!(Rotation::R0.upright((640, 480)), (640, 480));
}

#[test]
fn an_unturned_panel_is_blitted_as_it_always_was() {
    for window in [(720, 480), (640, 480), (1440, 960), (800, 480)] {
        for shake in [0.0, 6.0] {
            assert_eq!(
                panel_rect(window, Rotation::R0, shake),
                blit_rect(window, shake)
            );
        }
    }
}

#[test]
fn a_turned_panel_is_filled_by_the_picture_turned_to_fit_it() {
    let (w, h) = (out_w(), out_h());
    assert_eq!(
        panel_rect((h, w), Rotation::R90, 0.0),
        (0, 0, h as i32, w as i32)
    );
    assert_eq!(
        panel_rect((h, w), Rotation::R270, 0.0),
        (0, 0, h as i32, w as i32)
    );
    assert_eq!(
        panel_rect((w, h), Rotation::R180, 0.0),
        (0, 0, w as i32, h as i32)
    );
}

#[test]
fn a_shake_moves_the_picture_along_its_own_width_on_a_turned_panel() {
    let (w, h) = (out_w(), out_h());
    let still = panel_rect((2 * h, 2 * w), Rotation::R90, 0.0);
    let shaken = panel_rect((2 * h, 2 * w), Rotation::R90, 3.0);
    assert_eq!(
        still.0, shaken.0,
        "a shake across the picture moved it up the panel"
    );
    assert_ne!(still.1, shaken.1, "the shake was lost");
}

#[test]
fn each_turn_takes_the_pictures_top_edge_where_it_belongs() {
    let top_left = (0.0, 1.0);
    let (u, v) = sample(Rotation::R0, top_left);
    assert_eq!((u, v), (0.0, 1.0));
    let (u, v) = sample(Rotation::R90, top_left);
    assert_eq!(
        (u, v),
        (1.0, 1.0),
        "a counter-clockwise turn puts the top right there"
    );
    let (u, v) = sample(Rotation::R180, top_left);
    assert_eq!((u, v), (1.0, 0.0));
    let (u, v) = sample(Rotation::R270, top_left);
    assert_eq!(
        (u, v),
        (0.0, 0.0),
        "a clockwise turn puts the bottom left there"
    );
    for rot in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        assert_eq!(
            sample(rot, (0.5, 0.5)),
            (0.5, 0.5),
            "{rot:?} moved the centre"
        );
    }
}
