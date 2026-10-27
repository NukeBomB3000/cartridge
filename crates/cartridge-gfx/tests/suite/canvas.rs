use cartridge_gfx::{canvas, out_h, out_w, set_canvas, Canvas};

#[test]
fn a_canvas_is_named_by_its_size() {
    assert_eq!(Canvas::parse("720x480"), Some(Canvas::WIDE));
    assert_eq!(Canvas::parse(" 640X480\n"), Some(Canvas::FOUR_THREE));
    assert_eq!(Canvas::parse("720 x 720"), Some(Canvas::SQUARE));
    assert_eq!(
        Canvas::parse("800x600"),
        None,
        "only the canvases cartridge lays out"
    );
    assert_eq!(Canvas::parse("640"), None);
    assert_eq!(Canvas::parse(""), None);
}

#[test]
fn every_panel_cartridge_runs_on_gets_its_own_canvas() {
    assert_eq!(Canvas::for_panel(720, 480), Canvas::WIDE);
    assert_eq!(Canvas::for_panel(640, 480), Canvas::FOUR_THREE);
    assert_eq!(Canvas::for_panel(720, 720), Canvas::SQUARE);
}

#[test]
fn a_panel_of_another_size_gets_the_canvas_closest_to_its_shape() {
    assert_eq!(Canvas::for_panel(1280, 720), Canvas::WIDE);
    assert_eq!(Canvas::for_panel(1024, 768), Canvas::FOUR_THREE);
    assert_eq!(Canvas::for_panel(800, 800), Canvas::SQUARE);
    assert_eq!(Canvas::for_panel(0, 0), Canvas::SQUARE);
}

#[test]
fn the_canvas_cannot_change_once_something_has_drawn_at_it() {
    let current = canvas();
    assert_eq!((out_w(), out_h()), (current.w, current.h));
    assert_eq!(
        set_canvas(current),
        Ok(()),
        "asking again for the same one is fine"
    );
    let other = *Canvas::ALL.iter().find(|c| **c != current).unwrap();
    assert_eq!(set_canvas(other), Err(current));
    assert_eq!(canvas(), current);
}
