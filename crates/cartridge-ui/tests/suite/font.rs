use cartridge_ui::text::{self, label_font, on_grid, Layout, FONT_GRID};

#[test]
fn every_word_is_set_in_pixel_code() {
    let face = label_font().expect("no face");
    assert_eq!(face.units_per_em(), 1008.0, "not Pixel Code's em");
    let m = face.metrics('M', 18.0).advance_width;
    assert_eq!(
        face.metrics('i', 18.0).advance_width,
        m,
        "not a monospaced face"
    );
    assert!(
        (m - 12.0).abs() < 1e-3,
        "a letter is {m} wide at 18, not six of the face's pixels"
    );
}

/// Text lands on the art grid: a face pixel is a whole number of art pixels (18 px is one, 36
/// two), never half of one (9 px) or one and a half (27 px).
#[test]
fn sizes_land_on_the_faces_grid() {
    assert_eq!(FONT_GRID, 18.0);
    assert_eq!(on_grid(36.0), 36.0);
    assert_eq!(on_grid(19.0), 18.0);
    assert_eq!(on_grid(30.0), 36.0);
    assert_eq!(on_grid(27.0), 36.0, "27 px is an art pixel and a half");
    for small in [4.0, 9.0] {
        assert_eq!(
            on_grid(small),
            18.0,
            "nothing is set smaller than an art pixel to a face pixel"
        );
    }
    let face = label_font().unwrap();
    for (max, w) in [(40.0, 1000.0), (40.0, 150.0), (40.0, 60.0)] {
        let layout = text::fit(face, "Fast Forward", w, 1, max, 9.0);
        assert_eq!(
            layout.px % FONT_GRID,
            0.0,
            "fit chose {} px for {w}",
            layout.px
        );
        assert!(layout.px <= max);
    }
    assert_eq!(text::fit(face, "Rumble", 1000.0, 1, 40.0, 9.0).px, 36.0);
    assert_eq!(text::fit(face, "Rumble", 1000.0, 1, 30.0, 9.0).px, 18.0);
}

#[test]
fn a_letter_is_either_inked_or_not() {
    let layout = Layout {
        lines: vec!["Café Wi-Fi".into()],
        px: 27.0,
        tracking: 0.0,
    };
    let cov = text::coverage(200, 40, &layout);
    assert!(cov.contains(&255), "nothing was drawn");
    assert!(
        cov.iter().all(|&a| a == 0 || a == 255),
        "a soft edge: the face was not set on its grid"
    );
}

#[test]
fn what_the_face_lacks_is_spelled_out() {
    let face = label_font().unwrap();
    assert_eq!(text::printed(face, "\u{a9} 2026"), "(C) 2026");
    assert_eq!(
        text::printed(face, "Pok\u{e9}mon \u{2026} 6\u{d7}"),
        "Pok\u{e9}mon \u{2026} 6\u{d7}"
    );
    assert_eq!(
        text::line_width(face, "\u{a9}", 18.0, 0.0),
        text::line_width(face, "(C)", 18.0, 0.0),
        "measured as one thing, drawn as another"
    );
}
