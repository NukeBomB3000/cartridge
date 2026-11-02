use cartridge::hud_faces::{lit, render, ICONS, MAGIC};
use cartridge_ui::HudKind;

fn word(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
}

#[test]
fn the_file_says_how_big_a_face_is_and_holds_one_per_icon_and_lit_cell() {
    let data = render(0);
    assert_eq!(&data[..4], MAGIC);
    let (w, h, cells, faces) = (
        word(&data, 4),
        word(&data, 8),
        word(&data, 20),
        word(&data, 24),
    );
    assert_eq!(cells, HudKind::Volume.cells());
    assert_eq!(faces as usize, ICONS.len() * (cells as usize + 1));
    assert_eq!(data.len(), 28 + (faces * w * h * 4) as usize);
}

#[test]
fn a_face_with_more_cells_lit_differs_from_one_with_fewer() {
    let data = render(0);
    let size = (word(&data, 4) * word(&data, 8) * 4) as usize;
    let face = |n: usize| &data[28 + n * size..28 + (n + 1) * size];
    assert_ne!(face(0), face(20));
    assert_ne!(face(5), face(6));
    // the headphones' faces start where the speaker's 21 end
    assert_ne!(face(10), face(21 + 10));
}

#[test]
fn the_volume_lights_cells_as_the_gauge_does() {
    assert_eq!(lit(0), 0);
    assert_eq!(lit(100), 20);
    assert_eq!(lit(50), 10);
    assert_eq!(lit(12), 2);
}
