//! The PSP's UMD on the shelf. With UMD_OUT set, writes the faces there as PNGs to look at, each
//! at four times its size: a UMD with label art and one without.

use cartridge_store::{scan, Cart, Platform};
use cartridge_ui::{
    cart_face, label_ink, label_text, model_box, model_of, model_shadow, seated_box, shell_for,
    umd_silhouette, CartFace, CartModel, UMD_SHELL,
};
use tempfile::TempDir;

const ART: [u8; 3] = [0x20, 0xc0, 0x40];

/// A card with a UMD image for each of `stems`, and label art for those in `labelled`.
fn root(stems: &[&str], labelled: &[&str]) -> TempDir {
    let d = tempfile::tempdir().expect("tempdir");
    let games = d.path().join("Games/PSP");
    std::fs::create_dir_all(&games).unwrap();
    for stem in stems {
        std::fs::write(games.join(format!("{stem}.iso")), vec![0u8; 64 * 1024]).unwrap();
    }
    let labels = d.path().join("Labels/PSP");
    std::fs::create_dir_all(&labels).unwrap();
    for stem in labelled {
        let f = std::fs::File::create(labels.join(format!("{stem}.png"))).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(f), 40, 38);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let rgb: Vec<u8> = (0..40 * 38).flat_map(|_| ART).collect();
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

fn pixel(face: &CartFace, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * face.w + x) * 4) as usize;
    [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2]]
}

fn alpha(face: &CartFace, x: u32, y: u32) -> u8 {
    face.rgba[((y * face.w + x) * 4 + 3) as usize]
}

/// The box of the opaque pixels of `face` that are `colour`.
fn colour_box(face: &CartFace, colour: [u8; 3]) -> Option<(u32, u32, u32, u32)> {
    let mut b: Option<(u32, u32, u32, u32)> = None;
    for y in 0..face.h {
        for x in 0..face.w {
            if pixel(face, x, y) == colour && alpha(face, x, y) == 255 {
                b = Some(match b {
                    None => (x, y, x, y),
                    Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                });
            }
        }
    }
    b
}

#[test]
fn a_psp_game_stands_as_a_umd_the_real_disc_s_shape() {
    let d = root(&["Sega Rally Revo"], &[]);
    let carts = scan(d.path()).unwrap();
    let c = cart(&carts, "Sega Rally Revo");
    assert_eq!(c.platform, Platform::Psp);
    assert_eq!(model_of(c), CartModel::Umd);
    let (w, h) = model_box(CartModel::Umd);
    assert_eq!(seated_box(Platform::Psp), (w, h));
    let (aspect, real) = (w as f32 / h as f32, 64.0 / 63.2);
    assert!(
        (aspect / real - 1.0).abs() < 0.02,
        "the face is {aspect}:1, the real UMD {real}:1"
    );
    assert!(
        w % 4 == 0 && h % 2 == 0,
        "the face, {w}x{h}, does not halve onto the art grid"
    );
    let thick = CartModel::Umd.thick();
    assert!(
        thick > CartModel::HuCard.thick() && thick < CartModel::Gba.thick(),
        "a UMD is thicker than a card and thinner than a cart"
    );
}

#[test]
fn a_umd_face_is_a_dome_with_a_curved_foot_and_is_backed_by_its_own_shadow() {
    let d = root(&["Sega Rally Revo"], &["Sega Rally Revo"]);
    let carts = scan(d.path()).unwrap();
    let face = cart_face(cart(&carts, "Sega Rally Revo"));
    assert_eq!((face.w, face.h), model_box(CartModel::Umd));
    let mask = umd_silhouette(face.w, face.h);
    for (i, (px, cover)) in face.rgba.chunks_exact(4).zip(&mask).enumerate() {
        assert!(
            px[3] <= *cover,
            "pixel {i} is {} opaque outside the outline's {cover}",
            px[3]
        );
    }
    // the top is a half circle: the corners are clear well down the sides, the foot's too
    for (x, y) in [
        (0, 0),
        (face.w - 1, 0),
        (4, 30),
        (face.w - 5, 30),
        (0, face.h - 1),
        (face.w - 1, face.h - 1),
    ] {
        assert_eq!(alpha(&face, x, y), 0, "the corner at {x},{y} is not round");
    }
    // and the sides below the dome are straight and solid
    assert!(alpha(&face, 5, face.h / 2) > 250);
    assert!(alpha(&face, face.w - 6, face.h / 2) > 250);
    assert!(alpha(&face, face.w / 2, face.h / 2) > 250);
    let shadow = model_shadow(CartModel::Umd);
    for (px, cover) in shadow.rgba.chunks_exact(4).zip(&mask) {
        assert_eq!(&px[..3], &[0, 0, 0], "the shadow is not black");
        assert_eq!(px[3], *cover, "the shadow is not the shell's shape");
    }
}

#[test]
fn a_umd_label_is_a_round_window_across_nearly_all_the_face() {
    let d = root(&["Sega Rally Revo"], &["Sega Rally Revo"]);
    let carts = scan(d.path()).unwrap();
    let face = cart_face(cart(&carts, "Sega Rally Revo"));
    let (x0, y0, x1, y1) = colour_box(&face, ART).expect("the label art was not pasted");
    let (lw, lh) = ((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32);
    let (fw, fh) = (face.w as f32, face.h as f32);
    assert!(
        lw / fw > 0.82 && (lw - lh).abs() <= 1.0,
        "the label is {lw}x{lh} on a {fw}x{fh} face, not a circle across it"
    );
    assert!(
        ((x0 + x1 + 1) as f32 / 2.0 - fw / 2.0).abs() <= 1.0,
        "the label is not across the middle"
    );
    // a circle: its corners are not label
    for (x, y) in [
        (x0 + 2, y0 + 2),
        (x1 - 2, y0 + 2),
        (x0 + 2, y1 - 2),
        (x1 - 2, y1 - 2),
    ] {
        assert_ne!(
            pixel(&face, x, y),
            ART,
            "the label's corner at {x},{y} is square"
        );
    }
    assert_eq!(pixel(&face, (x0 + x1) / 2, (y0 + y1) / 2), ART);
}

#[test]
fn a_umd_with_no_label_art_is_labelled_with_its_name() {
    let d = root(&["Sega Rally Revo"], &[]);
    let carts = scan(d.path()).unwrap();
    let c = cart(&carts, "Sega Rally Revo");
    let face = cart_face(c);
    let ink = label_ink(&label_text(c));
    let (x0, y0, x1, y1) = colour_box(&face, ink).expect("no generated label");
    let inked = (y0..=y1)
        .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
        .filter(|&(x, y)| pixel(&face, x, y) == ink)
        .count();
    assert!(inked > 100, "no title on its label");
}

#[test]
fn a_umd_shell_is_pale() {
    let d = root(&["Sega Rally Revo"], &[]);
    let carts = scan(d.path()).unwrap();
    assert_eq!(shell_for(cart(&carts, "Sega Rally Revo")), UMD_SHELL);
    assert!(UMD_SHELL.colour.iter().all(|c| *c > 0xb0));
}

#[test]
fn render_umds() {
    let Ok(out) = std::env::var("UMD_OUT") else {
        return;
    };
    let stems = ["Sega Rally Revo", "Final Fantasy"];
    let d = root(&stems, &["Sega Rally Revo"]);
    let carts = scan(d.path()).unwrap();
    for stem in stems {
        let face = cart_face(cart(&carts, stem));
        let zoom = 4;
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
