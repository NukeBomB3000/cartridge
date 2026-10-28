use cartridge_store::Theme;
use cartridge_ui::{
    edge, housing, link_face, set_theme, ChromeColours, Draw, Link, Links, Printed, TopBar,
    HUD_INK, LINK_OFF, LINK_ON,
};

#[test]
fn the_card_s_palette_is_what_reaches_the_screen() {
    let t = Theme {
        housing: [0x11, 0x22, 0x33],
        edge: [0xaa, 0xbb, 0xcc],
    };
    // the widgets that still draw in the theme's greys take it from the one set at boot
    set_theme(t);
    let hex = |c: [f32; 4]| {
        [
            (c[0] * 255.0).round() as u8,
            (c[1] * 255.0).round() as u8,
            (c[2] * 255.0).round() as u8,
        ]
    };
    assert_eq!(hex(housing()), t.housing);
    assert_eq!(hex(edge()), t.edge);

    // the top bar is handed it, as the app hands it a card's theme over the palette
    let themed = ChromeColours::themed(&t);
    let mut out = Vec::new();
    TopBar {
        colours: themed,
        links: Links::default(),
        faces: None,
        clock: Printed::default(),
        battery: None,
        percent: Printed::default(),
        bolt: None,
    }
    .draw(&mut out);
    // the theme still names the housing and the edge, the bar draws neither of them now
    for (c, set, name) in [
        (themed.housing, housing(), "housing"),
        (themed.edge, edge(), "edge"),
    ] {
        assert_eq!(c, set, "the top bar's {name} is not the theme's");
    }
    assert!(
        out.iter().all(|d| match d {
            Draw::Rect { colour, .. } => colour[3] == 0.0,
            _ => true,
        }),
        "the top bar draws a ground behind its icons"
    );
}

/// A theme.txt names the chrome's greys and no ink, so the top bar keeps the white and grey it
/// was printed in, whatever colour the player picked.
#[test]
fn a_theme_keeps_the_top_bar_white() {
    let t = Theme {
        housing: [0x11, 0x22, 0x33],
        ..Theme::default()
    };
    let themed = ChromeColours::themed(&t);
    assert_eq!((themed.ink, themed.ink_off), (LINK_ON, LINK_OFF));
    assert_eq!(LINK_ON, HUD_INK, "the theme's ink is not the white it was");
    assert_eq!(ChromeColours::default().ink, LINK_ON);
    let amber = cartridge_store::colour_index("amber").expect("a backdrop colour");
    let palette = cartridge_ui::kit::palette::palette(amber);
    assert_ne!(ChromeColours::of(palette).ink, themed.ink);
    let wifi = link_face(Link::Wifi, true, &themed);
    assert!(
        wifi.rgba
            .chunks(4)
            .filter(|p| p[3] > 200)
            .all(|p| [p[0], p[1], p[2]] == HUD_INK),
        "the theme's Wi-Fi is not white"
    );
}
