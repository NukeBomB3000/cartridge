//! Sega's 8-bit carts on the shelf: the Game Gear's and the Master System's. With SEGA8_OUT
//! set, writes the faces there as PNGs to look at, each at three times its size.

use omega_store::{scan, Cart, Platform};
use omega_ui::{
    cart_face, model_box, model_of, sega8_silhouette, shell_for, CartFace, CartModel, Sega8Shell,
    GAME_GEAR_SHELL, MASTER_SYSTEM_SHELL,
};
use tempfile::TempDir;

const ART: [u8; 3] = [0x20, 0xc0, 0x40];

/// A card with a blank cart for each of `games` under its system's folder, label art for each.
fn root(games: &[(Platform, &str)]) -> TempDir {
    let d = tempfile::tempdir().expect("tempdir");
    for (platform, stem) in games {
        let dir = d.path().join("Games").join(platform.dir_name());
        std::fs::create_dir_all(&dir).unwrap();
        let ext = platform.extensions()[0];
        std::fs::write(dir.join(format!("{stem}.{ext}")), vec![0u8; 0x8000]).unwrap();
        let labels = d.path().join("Labels").join(platform.dir_name());
        std::fs::create_dir_all(&labels).unwrap();
        let f = std::fs::File::create(labels.join(format!("{stem}.png"))).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(f), 40, 36);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let rgb: Vec<u8> = (0..40 * 36).flat_map(|_| ART).collect();
        enc.write_header().unwrap().write_image_data(&rgb).unwrap();
    }
    d
}

fn cart<'a>(carts: &'a [Cart], stem: &str) -> &'a Cart {
    carts
        .iter()
        .find(|c| c.stem == stem)
        .unwrap_or_else(|| panic!("{stem} was not scanned"))
}

fn alpha(face: &CartFace, x: u32, y: u32) -> u8 {
    face.rgba[((y * face.w + x) * 4 + 3) as usize]
}

#[test]
fn a_game_gear_game_stands_as_a_game_gear_cart_and_a_master_system_game_as_its_own() {
    let d = root(&[
        (Platform::Gg, "Sonic Chaos (Europe)"),
        (Platform::Sms, "Alex Kidd (Europe)"),
    ]);
    let carts = scan(d.path()).unwrap();
    let gg = cart(&carts, "Sonic Chaos (Europe)");
    let sms = cart(&carts, "Alex Kidd (Europe)");
    assert_eq!(model_of(gg), CartModel::Sega8(Sega8Shell::GameGear));
    assert_eq!(model_of(sms), CartModel::Sega8(Sega8Shell::MasterSystem));
    assert_eq!(shell_for(gg), GAME_GEAR_SHELL);
    assert_eq!(shell_for(sms), MASTER_SYSTEM_SHELL);
}

#[test]
fn the_mega_cartridge_is_wider_than_tall_and_the_mark_iii_cart_taller_than_wide() {
    let (w, h) = model_box(CartModel::Sega8(Sega8Shell::MasterSystem));
    assert!(w > h, "{w}x{h}");
    let (w, h) = model_box(CartModel::Sega8(Sega8Shell::MarkIii));
    assert!(h > w, "{w}x{h}");
}

#[test]
fn a_japanese_master_system_game_stands_as_a_mark_iii_cart() {
    let d = root(&[(Platform::Sms, "Alex Kidd no Miracle World (Japan)")]);
    let carts = scan(d.path()).unwrap();
    assert_eq!(
        model_of(cart(&carts, "Alex Kidd no Miracle World (Japan)")),
        CartModel::Sega8(Sega8Shell::MarkIii)
    );
}

#[test]
fn the_game_gear_cart_has_its_top_corners_cut_off() {
    let (w, h) = model_box(CartModel::Sega8(Sega8Shell::GameGear));
    let mask = sega8_silhouette(Sega8Shell::GameGear, w, h);
    assert_eq!(mask[0], 0, "the top left corner is not cut");
    assert_eq!(mask[(w - 1) as usize], 0, "the top right corner is not cut");
    assert_eq!(mask[((h / 2) * w + w / 2) as usize], 255);
}

#[test]
fn each_face_wears_its_label_inside_its_outline() {
    let d = root(&[
        (Platform::Gg, "Columns (Japan)"),
        (Platform::Sms, "Wonder Boy (USA)"),
    ]);
    let carts = scan(d.path()).unwrap();
    for stem in ["Columns (Japan)", "Wonder Boy (USA)"] {
        let face = cart_face(cart(&carts, stem));
        assert_eq!(alpha(&face, face.w / 2, face.h / 2), 255, "{stem}");
        assert_eq!(alpha(&face, 0, 0), 0, "{stem}: its corner is not cut away");
    }
}

#[test]
fn render_sega8_carts() {
    let Ok(out) = std::env::var("SEGA8_OUT") else {
        return;
    };
    let games = [
        (Platform::Gg, "Sonic Chaos (Europe)"),
        (Platform::Sms, "Alex Kidd in Miracle World (Europe)"),
        (Platform::Sms, "Alex Kidd no Miracle World (Japan)"),
    ];
    let d = root(&games);
    let carts = scan(d.path()).unwrap();
    for (_, stem) in games {
        let face = cart_face(cart(&carts, stem));
        let zoom = 3;
        let (w, h) = (face.w * zoom, face.h * zoom);
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let s = (((y / zoom) * face.w + x / zoom) * 4) as usize;
                let d = ((y * w + x) * 4) as usize;
                px[d..d + 4].copy_from_slice(&face.rgba[s..s + 4]);
            }
        }
        let path = std::path::Path::new(&out).join(format!("{stem}.png"));
        let f = std::fs::File::create(path).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&px).unwrap();
    }
}
