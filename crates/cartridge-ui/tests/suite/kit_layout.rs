use cartridge_gfx::{out_h, out_w};
use cartridge_ui::kit::layout::{centred, modal_rect, page, panel_height, BAR_U, MODAL_MARGIN};

#[test]
fn the_page_is_the_screen_above_the_bar_in_art_pixels() {
    assert_eq!(page(), [out_w() / 2, out_h() / 2 - BAR_U]);
}

#[test]
fn a_modal_keeps_its_margin_inside_the_page() {
    let [w, h] = page();
    let [top, side, bottom] = MODAL_MARGIN;
    let [x, y, mw, mh] = modal_rect();
    assert_eq!((x, y), (side, top));
    assert_eq!(x + mw + side, w);
    assert_eq!(y + mh + bottom, h);
    if out_w() == 640 && out_h() == 480 {
        assert_eq!([mw, mh], [288, 206], "flow.html's modal at 640x480");
    }
}

#[test]
fn a_lone_panel_sits_in_the_middle_rounded_down() {
    let [w, h] = page();
    let [x, y, pw, ph] = centred(201, 101);
    assert_eq!((pw, ph), (201, 101));
    assert_eq!(x, (w - 201) / 2);
    assert_eq!(y, (h - 101) / 2);
}

#[test]
fn a_panel_is_its_content_and_its_chrome() {
    // a 160x60 panel's well is 37 tall (see kit.rs), so its chrome is 23
    assert_eq!(panel_height(37), 60);
}
