use cartridge_gfx::{blit_rect, blit_rect_fit, fit_rect, fit_scale, out_h, out_w};

fn canvas() -> (u32, u32) {
    (out_w(), out_h())
}

#[test]
fn integer_scale_never_fractional() {
    let (w, h) = canvas();
    assert_eq!(fit_scale(2 * w, 2 * h), 2);
    assert_eq!(fit_scale(2 * w + 60, 2 * h + 40), 2);
    assert_eq!(fit_scale(w - 20, h - 80), 1);
    assert_eq!(fit_scale(3 * w, 3 * h), 3);
}

#[test]
fn integer_scale_is_limited_by_the_tighter_axis() {
    let (w, h) = canvas();
    assert_eq!(fit_scale(3 * w, 2 * h), 2);
    assert_eq!(fit_scale(2 * w, 3 * h), 2);
}

#[test]
fn blit_rect_centres_the_scaled_output_in_the_window() {
    let (w, h) = canvas();
    let (dw, dh) = (2 * w as i32, 2 * h as i32);
    assert_eq!(fit_rect(2 * w, 2 * h), (0, 0, dw, dh));
    assert_eq!(fit_rect(2 * w + 60, 2 * h + 40), (30, 20, dw, dh));
    assert_eq!(fit_rect(3 * w, 2 * h), (w as i32 / 2, 0, dw, dh));
}

#[test]
fn blit_rect_overflows_symmetrically_when_the_window_is_too_small() {
    let (w, h) = canvas();
    assert_eq!(fit_rect(w - 20, h - 80), (-10, -40, w as i32, h as i32));
}

#[test]
fn a_panel_smaller_than_the_composite_fits_rather_than_crops() {
    let (w, h) = canvas();
    let panel = (w - 80, h);
    let (x, y, fw, fh) = blit_rect_fit(panel, 0.0);
    assert!(
        fw <= panel.0 as i32 && fh <= panel.1 as i32,
        "{fw}x{fh} does not fit a {}x{} panel",
        panel.0,
        panel.1
    );
    assert!(
        (fw as f32 / fh as f32 - w as f32 / h as f32).abs() < 0.01,
        "aspect was not preserved"
    );
    assert!(x >= 0 && y >= 0);
}

#[test]
fn the_device_panel_takes_the_fit_and_a_desktop_window_does_not() {
    let (w, h) = canvas();
    let panel = (w - 80, h);
    assert_eq!(blit_rect(panel, 0.0), blit_rect_fit(panel, 0.0));
    let window = (2 * w + 60, 2 * h + 40);
    assert_eq!(blit_rect(window, 0.0), fit_rect(window.0, window.1));
}

#[test]
fn the_fit_is_centred_and_shakes_with_the_picture() {
    let (w, h) = canvas();
    let panel = (w + 80, h);
    let (x, y, fw, fh) = blit_rect_fit(panel, 0.0);
    assert_eq!((x, fw), (40, w as i32));
    assert_eq!((y, fh), (0, h as i32));
    assert!(blit_rect_fit(panel, 6.0).0 > x);
}
