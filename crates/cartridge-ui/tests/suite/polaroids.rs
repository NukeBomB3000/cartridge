use std::path::Path;

use cartridge_store::StateEntry;
use cartridge_ui::{photo_face, PhotoFace, Polaroids, PHOTO_H, PHOTO_W};

fn entry(dir: &Path, stamp: &str) -> StateEntry {
    StateEntry {
        stamp: stamp.to_string(),
        state: dir.join(format!("{stamp}.state")),
        thumb: dir.join(format!("{stamp}.png")),
    }
}

fn write_png(path: &Path, w: u32, h: u32, rgb: [u8; 3]) {
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().expect("png header");
    let pixels: Vec<u8> = std::iter::repeat_n(rgb, (w * h) as usize)
        .flatten()
        .collect();
    writer.write_image_data(&pixels).expect("png data");
}

fn pixel(face: &PhotoFace, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * face.w + x) * 4) as usize;
    [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2]]
}

#[test]
fn relative_time_reads_off_the_stamp() {
    let now = "2026-08-09_14-36-05";
    assert_eq!(
        Polaroids::relative_time("2026-08-09_14-32-05", now),
        "4 min ago"
    );
    assert_eq!(
        Polaroids::relative_time("2026-08-09_13-36-05", now),
        "1 hr ago"
    );
    assert_eq!(
        Polaroids::relative_time("2026-08-09_11-36-05", now),
        "3 hr ago"
    );
}

#[test]
fn the_panel_says_the_age_with_a_capital() {
    let now = "2026-08-09_14-36-05";
    let age = |stamp| Polaroids::age_of(stamp, now);
    assert_eq!(age("2026-08-09_14-36-00"), "Just now");
    assert_eq!(age("2026-08-09_14-32-05"), "4 min ago");
    assert_eq!(age("2026-08-08_20-00-00"), "2026-08-08 20:00");
}

#[test]
fn every_save_has_its_age_newest_first() {
    let d = Path::new("/nonexistent");
    let mut p = Polaroids::new(vec![
        entry(d, "2026-08-09_14-36-00"),
        entry(d, "2026-08-09_14-26-05"),
    ]);
    let now = "2026-08-09_14-36-05";
    assert_eq!(p.ages(now), ["Just now", "10 min ago"]);
    assert_eq!(p.age(now), "Just now");
    p.right();
    assert_eq!(
        p.age(now),
        "10 min ago",
        "the age did not follow the selection"
    );
}

#[test]
fn an_hour_before_midnight_is_not_yesterday() {
    assert_eq!(
        Polaroids::relative_time("2026-08-08_23-30-00", "2026-08-09_01-00-00"),
        "1 hr ago"
    );
}

#[test]
fn a_stamp_ahead_of_the_clock_does_not_read_as_a_lifetime_ago() {
    assert_eq!(
        Polaroids::relative_time("2026-08-09_15-00-00", "2026-08-09_14-36-05"),
        "just now"
    );
}

#[test]
fn an_entry_whose_thumbnail_is_missing_still_has_a_face() {
    let face = photo_face(&entry(Path::new("/nonexistent"), "2026-08-09_14-32-05"));
    assert_eq!((face.w, face.h), (PHOTO_W, PHOTO_H));
    assert!(
        face.rgba.chunks_exact(4).all(|p| p[3] == 255),
        "the blank photo is see through"
    );
}

#[test]
fn the_photo_fills_the_face_and_keeps_its_colours() {
    let d = tempfile::tempdir().expect("tempdir");
    let e = entry(d.path(), "2026-08-09_14-32-05");
    write_png(&e.thumb, PHOTO_W, PHOTO_H, [0xd0, 0x40, 0x20]);
    let face = photo_face(&e);
    for (x, y) in [
        (0, 0),
        (PHOTO_W / 2, PHOTO_H / 2),
        (PHOTO_W - 1, PHOTO_H - 1),
    ] {
        assert_eq!(pixel(&face, x, y), [0xd0, 0x40, 0x20], "at {x},{y}");
    }
}

#[test]
fn the_index_clamps_at_both_ends() {
    let d = Path::new("/nonexistent");
    let mut p = Polaroids::new(vec![
        entry(d, "2026-08-09_00-00-02"),
        entry(d, "2026-08-09_00-00-01"),
    ]);
    p.left();
    assert_eq!(p.index, 0);
    p.right();
    p.right();
    p.right();
    assert_eq!(p.index, 1);
    assert_eq!(
        p.selected().map(|e| e.stamp.as_str()),
        Some("2026-08-09_00-00-01")
    );
}

#[test]
fn a_console_screenshot_keeps_its_own_size() {
    let d = tempfile::tempdir().expect("tempdir");
    for (w, h) in [(256, 224), (320, 240), (640, 480)] {
        let e = entry(d.path(), "2026-08-09_14-32-05");
        write_png(&e.thumb, w, h, [0x20, 0x90, 0xd0]);
        let face = photo_face(&e);
        assert_eq!(
            (face.w, face.h),
            (w, h),
            "the screenshot was not kept at its size"
        );
        assert_eq!(pixel(&face, w - 1, h - 1), [0x20, 0x90, 0xd0]);
    }
}
