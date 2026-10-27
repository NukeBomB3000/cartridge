//! The colours of the shelf's chrome, the bar across the top, from the kit's palette.

use cartridge_ui::kit::palette::palette;
use cartridge_ui::ChromeColours;

/// Each backdrop colour's chrome, by name.
fn chrome_in(name: &str) -> ChromeColours {
    let colour = cartridge_store::colour_index(name).expect("a backdrop colour");
    ChromeColours::of(palette(colour))
}

fn byte(c: [f32; 4]) -> [u8; 3] {
    [0, 1, 2].map(|i| (c[i] * 255.0).round() as u8)
}

/// As flow.html's shelf: the top bar in the well colour ruled in the edge one, printed in cream,
/// and what is off in the plate colour.
#[test]
fn the_chrome_is_drawn_in_the_kits_palette() {
    let (night, amber) = (chrome_in("night"), chrome_in("amber"));
    assert_ne!(night, amber, "the chrome looks the same in every colour");
    for name in cartridge_store::BACKDROP_COLOURS {
        let pal = palette(cartridge_store::colour_index(name).expect("a backdrop colour"));
        let c = chrome_in(name);
        assert_eq!(
            [c.housing, c.edge].map(byte),
            [pal.well, pal.edge],
            "{name}: the top bar is not in the well colour ruled in the edge one"
        );
        assert_eq!(
            (c.ink, c.ink_off),
            (pal.cream, pal.plate),
            "{name}: the top bar does not print in cream, and what is off in the plate colour"
        );
        assert!(
            c.housing[3] == 1.0 && c.edge[3] == 1.0,
            "{name}: the top bar lets the shelf show through it"
        );
    }
}

#[test]
fn night_keeps_the_chrome_in_greys_near_the_ones_it_had() {
    let (night, before) = (chrome_in("night"), ChromeColours::default());
    let grey = |c: [f32; 4]| {
        let [r, g, b] = byte(c);
        r == g && g == b
    };
    for (part, now, was) in [
        ("housing", night.housing, before.housing),
        ("edge", night.edge, before.edge),
    ] {
        assert!(grey(now), "Night's {part} is not grey: {:?}", byte(now));
        let off: i32 = (0..3)
            .map(|i| (byte(now)[i] as i32 - byte(was)[i] as i32).abs())
            .sum();
        assert!(
            off <= 60,
            "Night's {part} {:?} is far from the {:?} it was",
            byte(now),
            byte(was)
        );
    }
}

#[test]
fn every_colours_top_bar_is_ruled_in_a_colour_of_its_own() {
    for name in cartridge_store::BACKDROP_COLOURS {
        let c = chrome_in(name);
        assert!(
            byte(c.edge) != byte(c.housing),
            "{name}: the top bar's rule {:?} vanishes into the bar {:?}",
            byte(c.edge),
            byte(c.housing)
        );
    }
}
