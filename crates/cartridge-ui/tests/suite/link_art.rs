//! The link screen's art against assets/ui/components/cartridge-link.js: each piece held to a Chrome
//! screenshot of it (kit/ref/link-*.png, made by kit/make-refs.sh), pixel for pixel, in kit.rs's
//! RAMP; and what the screen needs of them, whatever the palette.

use crate::kit::{matches, RAMP};

use cartridge_ui::kit::palette::palette;
use cartridge_ui::link_art::{
    adapter_face, arc_face, cable_face, clicks_face, link_art, plug_face, port_face, End, Side,
    ADAPTER_H, ADAPTER_W, CLICKS_H, CLICKS_W, CONNECTOR_H, NOTCH_W, PLUG_H, TIP_H, TIP_W,
};
use cartridge_ui::UndoFace;

fn px(face: &UndoFace, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * face.w + x) * 4) as usize;
    [
        face.rgba[at],
        face.rgba[at + 1],
        face.rgba[at + 2],
        face.rgba[at + 3],
    ]
}

/// `face`'s pixels that are not clear.
fn inked(face: &UndoFace) -> usize {
    face.rgba.chunks_exact(4).filter(|p| p[3] > 0).count()
}

#[test]
fn the_plugs_are_chromes() {
    matches(&plug_face(End::Host, &RAMP), "link-plug-host");
    matches(&plug_face(End::Join, &RAMP), "link-plug-join");
}

#[test]
fn the_cable_is_chromes() {
    let row = cable_face(&RAMP);
    let mut cable = UndoFace {
        rgba: Vec::new(),
        w: row.w,
        h: 4,
    };
    for _ in 0..4 {
        cable.rgba.extend_from_slice(&row.rgba);
    }
    matches(&cable, "link-cable");
}

#[test]
fn the_adapter_is_chromes_cartridge_on_its_label_and_all() {
    matches(&adapter_face(&RAMP), "link-adapter");
}

#[test]
fn the_port_is_chromes_at_640_and_720() {
    matches(&port_face(320, 36, &RAMP), "link-port-320");
    matches(&port_face(360, 36, &RAMP), "link-port-360");
}

#[test]
fn the_arcs_and_the_clicks_are_chromes() {
    for ring in 0..3 {
        matches(
            &arc_face(ring, Side::Right, &RAMP),
            &format!("link-arc-{ring}"),
        );
    }
    matches(&arc_face(2, Side::Left, &RAMP), "link-arc-2-left");
    matches(&clicks_face(&RAMP), "link-clicks");
}

#[test]
fn every_piece_is_the_size_the_screen_lays_it_out_by() {
    let a = link_art(palette(0), 320, 36);
    assert_eq!((a.plug_host.w, a.plug_host.h), (30, PLUG_H));
    assert_eq!((a.plug_join.w, a.plug_join.h), (36, PLUG_H));
    assert_eq!((a.adapter.w, a.adapter.h), (ADAPTER_W, ADAPTER_H));
    assert_eq!((a.port.w, a.port.h), (320, 36));
    assert_eq!((a.clicks.w, a.clicks.h), (CLICKS_W, CLICKS_H));
    for i in 0..3 {
        let (r, l) = (&a.arcs_right[i], &a.arcs_left[i]);
        assert_eq!((r.w, r.h), (l.w, l.h));
        for y in 0..r.h {
            for x in 0..r.w {
                assert_eq!(
                    px(r, x, y),
                    px(l, r.w - 1 - x, y),
                    "arc {i} is not mirrored"
                );
            }
        }
    }
}

#[test]
fn the_host_plug_is_purple_and_the_joiners_grey() {
    let middle = |face: &UndoFace| px(face, face.w / 2, 30);
    let host = middle(&plug_face(End::Host, palette(0)));
    let join = middle(&plug_face(End::Join, palette(0)));
    assert!(
        host[2] > host[1] + 30,
        "the host plug is not purple: {host:?}"
    );
    assert!(
        (join[0] as i32 - join[2] as i32).abs() < 12,
        "the joiner's plug is not grey: {join:?}"
    );
}

#[test]
fn a_plugs_tip_is_its_bottom_rows_in_the_middle_and_fits_the_port() {
    for end in [End::Host, End::Join] {
        let face = plug_face(end, palette(0));
        let left = (face.w - TIP_W) / 2;
        for y in face.h - TIP_H..face.h - 1 {
            assert_eq!(px(&face, left - 1, y)[3], 0, "{end:?}: the tip is wider");
            assert_eq!(px(&face, left, y)[3], 255, "{end:?}: the tip is narrower");
            assert_eq!(px(&face, left + TIP_W - 1, y)[3], 255);
            assert_eq!(px(&face, left + TIP_W, y)[3], 0);
        }
        let above = face.h - TIP_H - 1;
        assert_eq!(
            px(&face, 2, above)[3],
            0,
            "the housing's corner is not round"
        );
        assert_eq!(
            px(&face, face.w / 2, above)[3],
            255,
            "nothing above the tip"
        );
    }
    // the port's opening, two rows down where its sides stop leaning in, takes the whole tip
    let pal = palette(0);
    let port = port_face(320, 36, pal);
    let opening = (0..port.w)
        .filter(|&x| px(&port, x, 2)[..3] == pal.dark)
        .count() as u32;
    assert!(opening >= TIP_W, "the port is narrower than the tip");
    assert!(
        NOTCH_W > opening,
        "the opening does not narrow from its top"
    );
}

#[test]
fn the_plugs_and_the_adapter_keep_their_colours_whatever_the_palette() {
    for colour in 0..8 {
        let pal = palette(colour);
        assert_eq!(
            plug_face(End::Host, pal).rgba,
            plug_face(End::Host, palette(0)).rgba
        );
        assert_eq!(adapter_face(pal).rgba, adapter_face(palette(0)).rgba);
    }
}

#[test]
fn the_port_the_arcs_and_the_clicks_are_drawn_in_the_palette() {
    for colour in 0..8 {
        let pal = palette(colour);
        let port = port_face(320, 36, pal);
        assert_eq!(&px(&port, 0, 0)[..3], &pal.frame, "the strip's lit edge");
        assert_eq!(&px(&port, 0, 1)[..3], &pal.stripe, "the stripe under it");
        assert_eq!(&px(&port, 0, 20)[..3], &pal.well, "the strip");
        assert_eq!(&px(&port, 160, 0)[..3], &pal.dark, "the port's opening");
        assert_eq!(&px(&arc_face(0, Side::Right, pal), 0, 0)[..3], &pal.cream);
        assert_eq!(&px(&clicks_face(pal), 3, 0)[..3], &pal.cream);
    }
}

#[test]
fn the_adapters_connector_is_its_bottom_rows_and_its_label_is_printed() {
    let face = adapter_face(palette(0));
    let body = face.h - CONNECTOR_H;
    for y in body..face.h {
        assert_eq!(
            px(&face, 2, y)[3],
            0,
            "the connector is as wide as the body"
        );
        assert_eq!(px(&face, face.w / 2, y)[3], 255, "no connector");
    }
    // CART's ink is the lightest of the adapter's greys, on the label plate
    let light = px(&face, 0, 0);
    let plate: Vec<[u8; 4]> = (24..35)
        .flat_map(|y| (24..55).map(move |x| (x, y)))
        .map(|(x, y)| px(&face, x, y))
        .collect();
    let brightest = plate.iter().map(|p| p[0]).max().unwrap_or(0);
    assert!(brightest > 0x90, "CART is not on the label: {light:?}");
    assert!(inked(&face) > (ADAPTER_W * (ADAPTER_H - CONNECTOR_H)) as usize / 2);
}

#[test]
fn every_face_is_opaque_or_clear() {
    let a = link_art(palette(7), 360, 36);
    for face in [
        &a.plug_host,
        &a.plug_join,
        &a.cable,
        &a.adapter,
        &a.port,
        &a.clicks,
        &a.arcs_right[0],
        &a.arcs_left[2],
    ] {
        assert!(
            face.rgba.chunks_exact(4).all(|p| p[3] == 0 || p[3] == 255),
            "a face is part see-through"
        );
    }
}
