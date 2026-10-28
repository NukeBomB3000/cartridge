use cartridge_power::{Battery, Charge};
use cartridge_ui::kit::palette::{palette, rgba, Palette};
use cartridge_ui::kit::pixel::{letters, ACCENT, ADVANCE, CAP};
use cartridge_ui::kit::text::{text_face, AIR, WORD_H};
use cartridge_ui::{
    bolt_face, link_face, out_w, ChromeColours, Draw, Link, LinkFaces, Links, Printed, TexId,
    TopBar, UndoFace, CELL, CELL_PX, FIGURE_AT, GAUGE_H, LINK_OFF, LINK_ON, SPRITE_PX, TOP_BAR_H,
};

const MARGIN: f32 = 24.0;

/// One art pixel on the screen.
const PX: f32 = SPRITE_PX as f32;

/// The clock's and the figure's faces: "16:00" and "100%", 6 art pixels a letter.
const CLOCK_W: u32 = 30;
const FIGURE_W: u32 = 24;

/// Every item's faces, in art pixels: the icons a cell square, ssh a word cut to its ink.
fn faces() -> LinkFaces {
    let pair = |base: usize, w: u32| {
        [
            (TexId::from_raw(base), w, CELL),
            (TexId::from_raw(base + 1), w, CELL),
        ]
    };
    LinkFaces {
        wifi: pair(10, CELL),
        bluetooth: pair(20, CELL),
        ssh: pair(30, 20),
    }
}

fn bar(links: Links, battery: Option<Battery>, bolt: Option<TexId>) -> Vec<Draw> {
    let faces = faces();
    let mut out = Vec::new();
    TopBar {
        colours: ChromeColours::default(),
        links,
        faces: Some(&faces),
        clock: Printed::new(TexId::from_raw(1), CLOCK_W),
        battery,
        percent: Printed::new(TexId::from_raw(2), FIGURE_W),
        bolt,
    }
    .draw(&mut out);
    out
}

fn battery(charge: Charge) -> Option<Battery> {
    Some(Battery {
        percent: 68,
        charge,
    })
}

fn tex_x(out: &[Draw], id: usize) -> Option<(f32, f32)> {
    out.iter().find_map(|d| match *d {
        Draw::Tex { x, w, tex, .. } if tex == TexId::from_raw(id) => Some((x, x + w)),
        _ => None,
    })
}

fn drawn(out: &[Draw], id: usize) -> bool {
    tex_x(out, id).is_some()
}

/// The clock's ink is in the middle, as near as whole art pixels allow, and the figure's ink
/// ends at the right margin, with the gauge in the room left of it.
#[test]
fn the_clock_sits_in_the_middle_and_the_battery_ends_at_the_right_margin() {
    let out = bar(Links::default(), battery(Charge::Discharging), None);
    let (a, b) = tex_x(&out, 1).expect("no clock");
    let ink_middle = (a + b - AIR as f32 * PX) / 2.0;
    assert!(
        (ink_middle - out_w() as f32 / 2.0).abs() <= PX / 2.0,
        "the clock's ink is centred on {ink_middle}"
    );
    let (figure, percent_end) = tex_x(&out, 2).expect("no figure");
    assert_eq!(
        percent_end - AIR as f32 * PX,
        out_w() as f32 - MARGIN,
        "the figure's ink does not end at the margin"
    );
    let leftmost_gauge = out
        .iter()
        .filter_map(|d| match *d {
            Draw::Rect { x, y, .. } if y > 0.0 && x > out_w() as f32 / 2.0 => Some(x),
            _ => None,
        })
        .fold(f32::MAX, f32::min);
    assert!(
        leftmost_gauge >= figure - FIGURE_AT,
        "the gauge starts at {leftmost_gauge}, left of the room it was given"
    );
}

/// Every face on the bar is drawn at two screen pixels to one of its art pixels, on even screen
/// pixels, so none of it is scaled smooth or set half an art pixel off.
#[test]
fn every_face_on_the_bar_is_drawn_whole_on_the_art_grid() {
    let out = bar(
        Links {
            wifi: true,
            bluetooth: Some(true),
            ssh: true,
        },
        battery(Charge::Charging),
        Some(TexId::from_raw(3)),
    );
    let faces = faces();
    let words = [(1, CLOCK_W, WORD_H), (2, FIGURE_W, WORD_H), (3, CELL, CELL)]
        .map(|(id, w, h)| (TexId::from_raw(id), w, h));
    let mut all = [faces.wifi, faces.bluetooth, faces.ssh].concat();
    all.extend(words);
    let size = |tex: TexId| -> (u32, u32) {
        let (_, w, h) = all.iter().find(|f| f.0 == tex).expect("a face");
        (*w, *h)
    };
    let mut drawn = 0;
    for d in &out {
        if let Draw::Tex {
            x, y, w, h, tex, ..
        } = *d
        {
            let (fw, fh) = size(tex);
            assert_eq!((w, h), (fw as f32 * PX, fh as f32 * PX), "{d:?} is scaled");
            assert!(
                [x, y].iter().all(|v| v.rem_euclid(PX) == 0.0),
                "{d:?} is off the art grid"
            );
            drawn += 1;
        }
    }
    assert_eq!(drawn, 6, "not every face drew: {out:?}");
}

#[test]
fn the_links_line_up_from_the_left_margin_wifi_then_bluetooth_then_ssh() {
    let out = bar(
        Links {
            wifi: true,
            bluetooth: Some(false),
            ssh: true,
        },
        None,
        None,
    );
    let wifi = tex_x(&out, 11).expect("Wi-Fi is not drawn on");
    let bluetooth = tex_x(&out, 20).expect("Bluetooth is not drawn off");
    let ssh = tex_x(&out, 31).expect("ssh is not drawn on");
    assert_eq!(wifi.0, MARGIN);
    assert!(wifi.1 < bluetooth.0 && bluetooth.1 < ssh.0, "out of order");
    for off in [10, 21, 30] {
        assert!(!drawn(&out, off), "face {off} is drawn in the wrong state");
    }
}

#[test]
fn every_link_shows_whether_it_is_on() {
    for (links, wanted) in [
        (Links::default(), [10, 30]),
        (
            Links {
                wifi: true,
                bluetooth: None,
                ssh: true,
            },
            [11, 31],
        ),
    ] {
        let out = bar(links, None, None);
        for id in wanted {
            assert!(drawn(&out, id), "{links:?} does not draw face {id}");
        }
    }
}

#[test]
fn a_handheld_without_bluetooth_shows_no_bluetooth_icon() {
    let out = bar(Links::default(), None, None);
    assert!(!drawn(&out, 20) && !drawn(&out, 21));
    let with = bar(
        Links {
            bluetooth: Some(false),
            ..Links::default()
        },
        None,
        None,
    );
    assert!(drawn(&with, 20), "an off Bluetooth radio is hidden");
}

#[test]
fn everything_on_the_bar_stays_inside_it() {
    let out = bar(
        Links {
            wifi: true,
            bluetooth: Some(true),
            ssh: true,
        },
        battery(Charge::Charging),
        Some(TexId::from_raw(3)),
    );
    for d in &out {
        if let Draw::Rect { y, h, .. } | Draw::Tex { y, h, .. } = *d {
            assert!(
                y >= 0.0 && y + h <= TOP_BAR_H,
                "{d:?} spills out of the bar"
            );
        }
    }
}

#[test]
fn charging_does_not_move_anything() {
    let idle = bar(Links::default(), battery(Charge::Discharging), None);
    let charging = bar(
        Links::default(),
        battery(Charge::Charging),
        Some(TexId::from_raw(3)),
    );
    for d in &idle {
        assert!(
            charging.contains(d),
            "{d:?} moved or vanished when charging started"
        );
    }
}

#[test]
fn a_bar_with_no_battery_still_has_its_clock() {
    let out = bar(Links::default(), None, None);
    assert!(drawn(&out, 1));
}

fn ink(face: &cartridge_ui::UndoFace) -> Option<[u8; 3]> {
    face.rgba
        .chunks(4)
        .find(|p| p[3] > 200)
        .map(|p| [p[0], p[1], p[2]])
}

#[test]
fn on_and_off_faces_are_the_same_size_and_tell_themselves_apart() {
    let white = ChromeColours::default();
    for link in Link::ALL {
        let (off, on) = (
            link_face(link, false, &white),
            link_face(link, true, &white),
        );
        assert!(on.w > 0 && on.h > 0, "{link:?} has no face");
        assert_eq!(
            (on.w, on.h),
            (off.w, off.h),
            "{link:?} reflows when it changes"
        );
        assert_eq!(ink(&on), Some(LINK_ON), "{link:?} on is not lit");
        assert_eq!(ink(&off), Some(LINK_OFF), "{link:?} off is not dimmed");
    }
    for link in [Link::Wifi, Link::Bluetooth] {
        let shape = |on: bool| -> Vec<u8> {
            let face = link_face(link, on, &white);
            face.rgba.chunks(4).map(|p| p[3]).collect()
        };
        assert_eq!(
            shape(true),
            shape(false),
            "{link:?} changes shape, not only ink, when it goes off"
        );
    }
}

fn inked_rows(face: &cartridge_ui::UndoFace) -> Vec<u32> {
    (0..face.h)
        .filter(|&y| (0..face.w).any(|x| face.rgba[((y * face.w + x) * 4 + 3) as usize] > 0))
        .collect()
}

#[test]
fn every_icon_is_one_square_cell_of_pixel_art() {
    let cell = CELL;
    let white = ChromeColours::default();
    for face in [
        link_face(Link::Wifi, true, &white),
        link_face(Link::Bluetooth, true, &white),
        bolt_face(LINK_ON),
    ] {
        assert_eq!((face.w, face.h), (cell, cell), "an icon is not one cell");
        assert!(
            face.rgba.chunks(4).all(|p| p[3] == 0 || p[3] == 255),
            "an icon has soft edges, not whole pixels"
        );
        let rows = inked_rows(&face);
        assert_eq!(
            (rows[0], rows[rows.len() - 1]),
            (0, cell - 1),
            "an icon leaves rows of its cell unused"
        );
    }
    assert_eq!(GAUGE_H, CELL_PX, "the battery is not as tall as the icons");
}

#[test]
fn words_stand_exactly_as_tall_as_the_icons() {
    let ssh = link_face(Link::Ssh, true, &ChromeColours::default());
    assert_eq!(ssh.h, CELL, "ssh is not as tall as the icons");
    for word in ["100%", "16:00"] {
        let rows = inked_rows(&text_face(word, LINK_ON));
        assert_eq!(
            (rows[0], rows[rows.len() - 1] + 1),
            (ACCENT as u32, (ACCENT + CAP) as u32),
            "{word}'s ink does not fill one cell below its cap top"
        );
        assert_eq!(CAP as u32, CELL, "the capitals are not a cell tall");
    }
}

#[test]
fn every_item_stands_in_the_same_band() {
    let out = bar(
        Links {
            wifi: true,
            bluetooth: Some(true),
            ssh: true,
        },
        battery(Charge::Charging),
        Some(TexId::from_raw(3)),
    );
    let top = |d: &Draw| match *d {
        Draw::Tex { y, tex, .. } if [1, 2].map(TexId::from_raw).contains(&tex) => {
            Some(y + ACCENT as f32 * PX)
        }
        Draw::Tex { y, h, .. } => Some(y + (h - CELL_PX) / 2.0),
        _ => None,
    };
    let tops: Vec<f32> = out.iter().filter_map(top).collect();
    assert_eq!(tops.len(), 6, "not every item drew: {out:?}");
    assert!(
        tops.iter().all(|&t| t == tops[0]),
        "items stand at different heights: {tops:?}"
    );
    let band = tops[0];
    assert_eq!(
        band + CELL_PX / 2.0,
        (TOP_BAR_H - 2.0) / 2.0,
        "the band is not centred above the rule"
    );
    let capsule_top = out
        .iter()
        .filter_map(|d| match *d {
            Draw::Rect { x, y, .. } if x > out_w() as f32 / 2.0 && y > 0.0 => Some(y),
            _ => None,
        })
        .fold(f32::MAX, f32::min);
    assert_eq!(capsule_top, band, "the battery does not stand in the band");
}

#[test]
fn the_bar_is_drawn_in_the_colours_it_is_handed() {
    let faces = faces();
    let bar_in = |name: &str| {
        let colour = cartridge_store::colour_index(name).expect("a backdrop colour");
        let colours = ChromeColours::of(cartridge_ui::kit::palette::palette(colour));
        let mut out = Vec::new();
        TopBar {
            colours,
            links: Links::default(),
            faces: Some(&faces),
            clock: Printed::new(TexId::from_raw(1), CLOCK_W),
            battery: None,
            percent: Printed::default(),
            bolt: None,
        }
        .draw(&mut out);
        let fills: Vec<[f32; 4]> = out
            .iter()
            .filter_map(|d| match *d {
                Draw::Rect { w, colour, .. } if w == out_w() as f32 => Some(colour),
                _ => None,
            })
            .collect();
        (fills, colours)
    };
    let (night, night_colours) = bar_in("night");
    let (amber, amber_colours) = bar_in("amber");
    // no housing and no rule: only the icons and the figures show, on an empty plate
    assert_eq!(night, [[0.0; 4]]);
    assert_eq!(amber, [[0.0; 4]]);
    assert_ne!(night_colours.ink, amber_colours.ink);
}

/// ssh is drawn bold the kit's way, not in Pixel Code's own bold: each letter of the plain face
/// drawn again a pixel to its right, every pixel solid.
#[test]
fn ssh_is_bolder_than_the_plain_face() {
    let face = link_face(Link::Ssh, true, &ChromeColours::default());
    let solid = face.rgba.chunks(4).filter(|p| p[3] == 255).count();
    let plain: usize = letters("ssh", ADVANCE)
        .iter()
        .map(|r| r.bytes().filter(|&b| b == b'#').count())
        .sum();
    assert!(
        solid > plain * 3 / 2,
        "{solid} solid pixels to the plain face's {plain}: ssh is not bold"
    );
    assert!(
        face.rgba.chunks(4).all(|p| p[3] == 0 || p[3] == 255),
        "ssh has soft edges"
    );
    assert!(face.h * SPRITE_PX <= TOP_BAR_H as u32);
}

fn colour(name: &str) -> &'static Palette {
    palette(cartridge_store::colour_index(name).expect("a backdrop colour"))
}

/// Every colour a face's solid pixels are in.
fn inks(face: &UndoFace) -> Vec<[u8; 3]> {
    let mut seen: Vec<[u8; 3]> = Vec::new();
    for p in face.rgba.chunks(4).filter(|p| p[3] > 200) {
        if !seen.contains(&[p[0], p[1], p[2]]) {
            seen.push([p[0], p[1], p[2]]);
        }
    }
    seen
}

fn coverage(face: &UndoFace) -> Vec<u8> {
    face.rgba.chunks(4).map(|p| p[3]).collect()
}

/// flow.html's shelf prints its top bar in --oz-cream; an item that is off takes --oz-plate,
/// as a network row draws the Wi-Fi bars a signal does not reach.
#[test]
fn the_bar_prints_in_the_palettes_cream_and_what_is_off_in_its_plate() {
    let mut seen = Vec::new();
    for name in ["night", "amber"] {
        let pal = colour(name);
        let colours = ChromeColours::of(pal);
        assert_eq!(
            (colours.ink, colours.ink_off),
            (pal.cream, pal.plate),
            "{name}"
        );
        for link in Link::ALL {
            let (on, off) = (
                link_face(link, true, &colours),
                link_face(link, false, &colours),
            );
            assert_eq!(inks(&on), [pal.cream], "{name}: {link:?} on is not cream");
            assert_eq!(inks(&off), [pal.plate], "{name}: {link:?} off is not plate");
        }
        assert_eq!(
            inks(&bolt_face(colours.ink)),
            [pal.cream],
            "{name}: the bolt"
        );
        let clock = text_face("16:00", colours.ink);
        assert_eq!(inks(&clock), [pal.cream], "{name}: the clock");
        assert_eq!(
            coverage(&clock),
            coverage(&text_face("16:00", LINK_ON)),
            "{name}: the clock changes shape with its ink"
        );

        let faces = faces();
        let mut out = Vec::new();
        TopBar {
            colours,
            links: Links::default(),
            faces: Some(&faces),
            clock: Printed::new(TexId::from_raw(1), CLOCK_W),
            battery: battery(Charge::Discharging),
            percent: Printed::new(TexId::from_raw(2), FIGURE_W),
            bolt: None,
        }
        .draw(&mut out);
        let capsule: Vec<[f32; 4]> = out
            .iter()
            .filter_map(|d| match *d {
                Draw::Rect { x, y, colour, .. } if y > 0.0 && x > out_w() as f32 / 2.0 => {
                    Some(colour)
                }
                _ => None,
            })
            .collect();
        assert!(!capsule.is_empty(), "{name}: no battery");
        assert!(
            capsule.iter().all(|&c| c == rgba(pal.cream, 1.0)),
            "{name}: the battery is not drawn in cream: {capsule:?}"
        );
        seen.push(colours.ink);
    }
    assert_ne!(seen[0], seen[1], "Night and Amber print the bar alike");
}

/// Night's ramp is grey, so the bar keeps the look it had in white and grey: its cream and
/// plate are each within a few steps of them, and what is off still sits between the bar and
/// what is on.
#[test]
fn night_prints_the_bar_close_to_the_white_and_grey_it_had() {
    let night = ChromeColours::of(colour("night"));
    let near = |a: [u8; 3], b: [u8; 3]| (0..3).all(|i| a[i].abs_diff(b[i]) <= 16);
    assert!(
        near(night.ink, LINK_ON),
        "{:?} is far from white",
        night.ink
    );
    assert!(
        near(night.ink_off, LINK_OFF),
        "{:?} is far from the grey",
        night.ink_off
    );
    let light = |c: [u8; 3]| c.iter().map(|&v| u32::from(v)).sum::<u32>();
    let housing = night.housing.map(|c| (c * 255.0).round() as u8);
    let housing = [housing[0], housing[1], housing[2]];
    assert!(
        light(housing) < light(night.ink_off) && light(night.ink_off) < light(night.ink),
        "what is off does not sit between the bar and what is on"
    );
}
