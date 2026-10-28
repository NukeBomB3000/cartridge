use std::time::Duration;

use cartridge::cheevos::overlay::{alpha, compose, lines, Face, Queue, FACE_H, FACE_W, SHOW};
use cartridge_cheevos::Notice;
use cartridge_ui::kit::palette::palette;

fn unlocked() -> Face {
    let notice = Notice::Unlocked {
        title: "First Blood".into(),
        description: "Defeat the first boss without taking any damage in the whole level".into(),
        points: 10,
        badge_url: String::new(),
    };
    Face {
        unlock: notice.is_unlock(),
        notice,
        badge: Some(vec![0xd0; 64 * 64 * 4]),
    }
}

#[test]
fn a_notice_is_one_panel_in_the_themes_colours() {
    let pal = palette(0);
    let face = compose(&unlocked(), pal);
    assert_eq!((face.w, face.h), (FACE_W, FACE_H));
    let px = |x: u32, y: u32| {
        let i = ((y * face.w + x) * 4) as usize;
        [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2]]
    };
    assert_eq!(px(0, 0), pal.frame, "its frame");
    assert_eq!(px(FACE_W - 2, FACE_H - 2), pal.dark, "its ground");
    assert_eq!(px(10, FACE_H / 2), [0xd0; 3], "the badge on its left");
    if let Ok(dir) = std::env::var("CHEEVOS_OUT") {
        let mut png = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut png, face.w, face.h);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()
                .unwrap()
                .write_image_data(&face.rgba)
                .unwrap();
        }
        std::fs::write(format!("{dir}/cheevos-notice.png"), png).unwrap();
    }
}

#[test]
fn a_notice_without_a_badge_still_composes() {
    let mut face = unlocked();
    face.badge = None;
    let out = compose(&face, palette(0));
    assert_eq!((out.w, out.h), (FACE_W, FACE_H));
}

#[test]
fn an_unlock_says_its_title_points_and_description() {
    let (head, detail) = lines(&unlocked().notice);
    assert_eq!(head, "First Blood - 10 pts");
    assert!(detail.starts_with("Defeat the first boss"));
}

#[test]
fn a_notice_fades_in_stays_and_fades_out() {
    assert_eq!(alpha(Duration::ZERO), Some(0.0));
    assert_eq!(alpha(SHOW / 2), Some(1.0));
    assert!(alpha(SHOW - Duration::from_millis(100)).unwrap() < 1.0);
    assert_eq!(alpha(SHOW), None);
}

#[test]
fn a_full_queue_drops_a_minor_notice_before_an_unlock() {
    let minor = || Face {
        notice: Notice::Online,
        badge: None,
        unlock: false,
    };
    let mut q = Queue::default();
    for _ in 0..8 {
        q.push(minor());
    }
    q.push(unlocked());
    assert_eq!(q.len(), 8);
    let mut unlocks = 0;
    while let Some(f) = q.pop() {
        unlocks += f.unlock as u32;
    }
    assert_eq!(unlocks, 1);
}
