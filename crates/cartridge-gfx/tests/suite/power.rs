use cartridge_gfx::{screen_brightness, screen_glow, screen_rect_in, screen_scale, screen_width};

#[test]
fn power_on_expands_from_a_line_to_the_full_frame() {
    assert!(screen_scale(0.0) < 0.02, "it starts at full height");
    assert!(screen_scale(0.5) > 0.2 && screen_scale(0.5) < 1.0);
    assert!(
        (screen_scale(1.0) - 1.0).abs() < 0.001,
        "it never reaches full height"
    );
}

#[test]
fn the_brightness_overshoots_before_it_settles() {
    let peak = (0..=100)
        .map(|i| screen_brightness(i as f32 / 100.0))
        .fold(0.0f32, f32::max);
    assert!(peak > 1.05, "no overshoot, peak {peak:.2}");
    assert!(
        (screen_brightness(1.0) - 1.0).abs() < 0.01,
        "it settles bright or dim"
    );
}

#[test]
fn the_line_closes_to_a_dot_at_the_very_end() {
    assert!(screen_width(0.0) < 0.02, "the line never closes to a dot");
    assert!(
        screen_width(0.5) > 0.99,
        "the collapse eats the whole animation"
    );
}

#[test]
fn the_picture_is_white_until_it_opens_and_fades_to_its_colours() {
    assert_eq!(screen_glow(0.0), 1.0, "the dot is not white");
    assert_eq!(screen_glow(0.3), 1.0, "the line is not white");
    let open = screen_glow(0.7);
    assert!(
        open > 0.0 && open < 0.5,
        "no afterglow once open: {open:.2}"
    );
    assert!(screen_glow(0.85) < open, "the afterglow does not fade");
    assert_eq!(screen_glow(1.0), 0.0, "the picture stays washed white");
    for i in 0..100 {
        let (a, b) = (i as f32 / 100.0, (i + 1) as f32 / 100.0);
        assert!(
            screen_glow(b) <= screen_glow(a),
            "the glow comes back at {b}"
        );
    }
}

#[test]
fn the_picture_opens_from_a_dot_to_a_line_to_all_of_it_on_the_grid() {
    // even both ways, so the dot and the line are two pixels in the very middle
    let r = [10.0, 20.0, 640.0, 428.0];
    let (_, _, w, h) = screen_rect_in(0.05, r);
    assert_eq!((w, h), (2.0, 2.0), "it does not start as a dot");
    let (_, _, w, h) = screen_rect_in(0.4, r);
    assert_eq!(w, 640.0, "the line is not across the whole picture");
    assert!(h < 428.0, "the picture opened before the line was across");
    assert_eq!(screen_rect_in(1.0, r), (10.0, 20.0, 640.0, 428.0));
    for i in 0..=100 {
        let (x, y, w, h) = screen_rect_in(i as f32 / 100.0, r);
        // the dot and the line, an art pixel thick, stand in the very middle instead
        if w > 2.0 {
            assert_eq!((x - r[0]) % 2.0, 0.0, "off the art grid across at {i}");
        }
        if h > 2.0 {
            assert_eq!((y - r[1]) % 2.0, 0.0, "off the art grid down at {i}");
        }
        assert_eq!(
            x - r[0],
            r[0] + r[2] - (x + w),
            "not in the middle across at {i}"
        );
        assert_eq!(
            y - r[1],
            r[1] + r[3] - (y + h),
            "not in the middle down at {i}"
        );
    }
}
