use cartridge_store::{Cart, Platform};
use cartridge_ui::{
    cart_mesh, gb_silhouette, holes, outline, posed_face, push_cart_side, silhouette,
    snes_silhouette, CartMeshes, CartModel, Draw, GbShell, MeshId, Pose, Shelf, SnesShell, TexId,
    ACTIVE_SCALE, CART_H, OUTLINE_INSET, THICK,
};

const DT: f32 = 1.0 / 60.0;

fn shelf_with(n: usize) -> Shelf {
    Shelf::new(
        (0..n)
            .map(|i| Cart {
                platform: Platform::Gba,
                stem: format!("Game {i}"),
                rom: format!("Games/GBA/Game {i}.gba").into(),
                label: None,
                code: String::new(),
                shell: None,
                title: format!("GAME {i}"),
            })
            .collect(),
    )
}

fn faced(n: usize) -> Shelf {
    let mut s = shelf_with(n);
    s.set_faces((0..n).map(|i| TexId::from_raw(40 + i)).collect());
    s
}

fn run(s: &mut Shelf, seconds: f32) {
    for _ in 0..(seconds / DT).round() as usize {
        s.update(DT);
    }
}

fn row(s: &Shelf) -> Vec<Draw> {
    let mut out = Vec::new();
    s.draw_row(None, 0.0, 0.0, 1.0, &mut out);
    out
}

#[test]
fn the_selected_cart_sways_twenty_degrees_either_way_and_floats_four_pixels() {
    let mut s = faced(3);
    run(&mut s, 2.0);
    let (mut yaw, mut lift) = ((f32::MAX, f32::MIN), (f32::MAX, f32::MIN));
    for _ in 0..8 * 60 {
        s.update(DT);
        let p = s.pose();
        yaw = (yaw.0.min(p.yaw), yaw.1.max(p.yaw));
        lift = (lift.0.min(p.lift), lift.1.max(p.lift));
    }
    let twenty = 20f32.to_radians();
    for (name, (low, high), most) in [("sway", yaw, twenty), ("float", lift, 4.0)] {
        assert!(
            high <= most * 1.0001 && high > most * 0.99,
            "the {name} reached {high} one way, for {most}"
        );
        assert!(
            low >= -most * 1.0001 && low < -most * 0.99,
            "the {name} reached {low} the other way, for {most}"
        );
    }
}

#[test]
fn a_shelf_that_has_just_stopped_eases_into_the_sway() {
    let mut s = faced(3);
    run(&mut s, 0.1);
    assert!(
        s.pose().yaw.abs() < 1f32.to_radians(),
        "the sway came in at full swing: {}",
        s.pose().yaw
    );
}

#[test]
fn the_sway_stops_while_the_shelf_scrolls_and_the_carts_lean_into_it() {
    let mut s = faced(5);
    run(&mut s, 3.0);
    s.right();
    run(&mut s, 0.15);
    let p = s.pose();
    assert!(
        p.yaw < -0.05,
        "carts sliding left did not lean into it: {}",
        p.yaw
    );
    assert_eq!(p.lift, 0.0, "the cart kept floating through the scroll");

    run(&mut s, 0.6);
    let spun: Vec<TexId> = row(&s)
        .iter()
        .filter_map(|d| match *d {
            Draw::Spun { tex, .. } => Some(tex),
            _ => None,
        })
        .collect();
    let selected = TexId::from_raw(40 + s.index);
    assert!(
        spun.iter().all(|t| *t == selected),
        "a cart beside the selected one still leans once the scroll has settled"
    );
}

#[test]
fn a_cart_still_waiting_for_its_face_stands_still() {
    let mut s = shelf_with(3);
    run(&mut s, 3.0);
    let out = row(&s);
    assert!(
        out.iter().all(|d| matches!(d, Draw::Rect { .. })),
        "a cart with no face turned or grew a side"
    );
    let middle = out
        .iter()
        .find_map(|d| match *d {
            Draw::Rect { y, h, .. } if (h - CART_H as f32 * ACTIVE_SCALE).abs() < 0.5 => Some(y),
            _ => None,
        })
        .expect("no cart at full size");
    assert_eq!(
        middle,
        cartridge_ui::shelf_cart().y,
        "a cart with no face floated"
    );
}

#[test]
fn held_still_the_selected_cart_settles_square_and_level_and_sways_again_once_let_go() {
    let mut s = faced(3);
    run(&mut s, 3.0);
    s.hold_still(true);
    run(&mut s, 0.2);
    assert_eq!(s.pose(), Pose::REST, "the cart kept swaying while held");
    s.hold_still(false);
    run(&mut s, 2.0);
    assert_ne!(s.pose(), Pose::REST, "the cart never swayed again");
}

#[test]
fn the_side_shows_on_the_side_that_turns_towards_the_eye() {
    let (x, y, w, h) = (100.0, 50.0, 360.0, 211.0);
    for (yaw, near_left) in [(0.17, true), (-0.17, false)] {
        let mut out = Vec::new();
        push_cart_side(
            &mut out,
            (x, y, w, h),
            TexId::from_raw(1),
            0.0,
            yaw,
            1.0,
            1.0,
        );
        assert!(!out.is_empty(), "a cart turned {yaw} has no side");
        let mut last = f32::MAX;
        for d in &out {
            let Draw::Spun {
                depth,
                clip,
                colour,
                yaw: drawn,
                ..
            } = *d
            else {
                panic!("the side is drawn with something other than spun slices: {d:?}");
            };
            assert_eq!(drawn, yaw);
            assert!(
                depth > 0.0 && depth <= THICK * w + 0.001 && depth < last,
                "the slices do not step forward from the back of the cart: {depth} after {last}"
            );
            last = depth;
            assert!(colour[0] < 1.0, "the side is as bright as the face");
            let [left, _, wide, _] = clip.expect("an upright side is not kept to its band");
            let inset = OUTLINE_INSET * w + 2.0;
            match near_left {
                true => assert!(
                    left < x && left + wide <= x + inset,
                    "turned {yaw}, the side's band {left}+{wide} is not on the left edge"
                ),
                false => assert!(
                    left >= x + w - inset && left + wide > x + w,
                    "turned {yaw}, the side's band {left}+{wide} is not on the right edge"
                ),
            }
        }
    }
}

#[test]
fn a_cart_turned_in_the_canvas_draws_its_side_whole() {
    let mut out = Vec::new();
    push_cart_side(
        &mut out,
        (100.0, 50.0, 360.0, 211.0),
        TexId::from_raw(1),
        0.2,
        0.17,
        1.0,
        1.0,
    );
    assert!(!out.is_empty());
    assert!(
        out.iter()
            .all(|d| matches!(d, Draw::Spun { clip: None, .. })),
        "a side that is not upright was cut to an upright band"
    );
}

#[test]
fn a_side_too_thin_to_see_is_not_drawn() {
    let mut out = Vec::new();
    push_cart_side(
        &mut out,
        (100.0, 50.0, 360.0, 211.0),
        TexId::from_raw(1),
        0.0,
        0.005,
        1.0,
        1.0,
    );
    assert!(out.is_empty(), "a side under half a pixel was drawn");
}

#[test]
fn an_unspun_face_is_drawn_as_it_always_was() {
    let (rect, tex) = ((1.0, 2.0, 3.0, 4.0), TexId::from_raw(7));
    assert_eq!(
        posed_face(rect, tex, 0.5, 0.0, 0.0),
        Draw::Tex {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
            tex,
            alpha: 0.5
        }
    );
    assert_eq!(
        posed_face(rect, tex, 0.5, 0.3, 0.0),
        Draw::Turned {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
            tex,
            alpha: 0.5,
            turn: 0.3
        }
    );
    assert!(matches!(
        posed_face(rect, tex, 0.5, 0.0, 0.1),
        Draw::Spun {
            yaw: 0.1,
            depth: 0.0,
            ..
        }
    ));
}

#[test]
fn every_side_of_a_cart_model_faces_out_of_the_cart() {
    for (name, model, (w, h), mask) in [
        ("GBA", CartModel::Gba, (240, 141), silhouette(240, 141)),
        (
            "GB",
            CartModel::Gb(GbShell::Notched),
            (240, 259),
            gb_silhouette(GbShell::Notched, 240, 259),
        ),
        (
            "GBC",
            CartModel::Gb(GbShell::Rounded),
            (240, 259),
            gb_silhouette(GbShell::Rounded, 240, 259),
        ),
        (
            "American SNES",
            CartModel::Snes(SnesShell::Ntsc),
            (544, 352),
            snes_silhouette(SnesShell::Ntsc, 544, 352),
        ),
        (
            "Super Famicom",
            CartModel::Snes(SnesShell::Famicom),
            (520, 344),
            snes_silhouette(SnesShell::Famicom, 520, 344),
        ),
    ] {
        let cover = |x: f32, y: f32| {
            let (x, y) = (x.round() as i32, y.round() as i32);
            match (0..w).contains(&x) && (0..h).contains(&y) {
                true => mask[(y * w + x) as usize],
                false => 0,
            }
        };
        let mesh = cart_mesh(model);
        let walls: Vec<_> = mesh
            .chunks_exact(6)
            .filter(|quad| quad[0].normal[2] == 0.0)
            .collect();
        assert!(
            walls.len() >= 12,
            "the {name} model has {} sides",
            walls.len()
        );
        for quad in walls {
            let (a, b) = (quad[0].xyz, quad[1].xyz);
            let (mx, my) = (
                (a[0] + b[0]) / 2.0 * w as f32,
                (a[1] + b[1]) / 2.0 * h as f32,
            );
            let [nx, ny, _] = quad[0].normal;
            assert!(
                cover(mx + 3.0 * nx, my + 3.0 * ny) < 64
                    && cover(mx - 3.0 * nx, my - 3.0 * ny) > 192,
                "the {name} side at ({mx}, {my}) faces ({nx}, {ny}), not out of the cart"
            );
            assert!(
                quad.iter()
                    .all(|v| v.xyz[2] == 0.0 || v.xyz[2] == model.thick()),
                "a {name} side does not run from the face back to the cart's thickness"
            );
        }
    }
}

#[test]
fn a_shelf_with_models_draws_each_cart_as_one_model_turned_as_it_is_held() {
    let mut s = faced(3);
    s.set_meshes(CartMeshes::build(|model| {
        MeshId::from_raw(model.index() + 1)
    }));
    run(&mut s, 2.6);
    let out = row(&s);
    assert!(
        !out.iter().any(|d| matches!(d, Draw::Spun { .. })),
        "a cart with a model still has its face spun flat and its side in slices"
    );
    let models: Vec<_> = out
        .iter()
        .filter_map(|d| match *d {
            Draw::Model {
                mesh, yaw, x, w, ..
            } => Some((mesh, yaw, x + w / 2.0)),
            _ => None,
        })
        .collect();
    assert_eq!(models.len(), 3, "three carts, not three models: {models:?}");
    assert!(models.iter().all(|(mesh, ..)| *mesh == MeshId::from_raw(1)));
    let middle = cartridge_ui::out_w() as f32 / 2.0;
    let (_, yaw, _) = models
        .iter()
        .min_by(|a, b| (a.2 - middle).abs().total_cmp(&(b.2 - middle).abs()))
        .copied()
        .unwrap();
    assert!(
        yaw != 0.0 && yaw == s.pose().yaw,
        "the middle cart is not held as it sways"
    );
}

#[test]
fn each_snes_cart_is_drawn_to_the_real_cart_s_size_and_thickness() {
    // the American cart is 136 by 88 mm and the Super Famicom's 130 by 86, both 20 mm thick:
    // drawn four units to the millimetre
    for (shell, (w, h)) in [
        (SnesShell::Ntsc, (136.0, 88.0)),
        (SnesShell::Famicom, (130.0, 86.0)),
    ] {
        let model = CartModel::Snes(shell);
        let (points, units) = outline(model);
        assert_eq!(units, (w * 4.0, h * 4.0), "the {shell:?} drawing");
        let span = |pick: fn(&(f32, f32)) -> f32| {
            let v: Vec<f32> = points.iter().map(pick).collect();
            v.iter().copied().fold(f32::MIN, f32::max) - v.iter().copied().fold(f32::MAX, f32::min)
        };
        assert_eq!(
            (span(|p| p.0), span(|p| p.1)),
            units,
            "the {shell:?} outline does not fill its drawing"
        );
        let thick = model.thick() * w;
        assert!(
            (thick - 20.0).abs() < 1.0,
            "the {shell:?} cart is {thick} mm thick, not 20"
        );
    }
}

/// The Super Famicom cart's top is straight across from corner to corner, its corners 3.5 mm
/// round in a few cuts, its sides straightening 3.5 mm down.
#[test]
fn the_super_famicom_cart_s_top_is_straight_with_its_corners_3_5_mm_round() {
    let (points, (w, _)) = outline(CartModel::Snes(SnesShell::Famicom));
    let mm = 4.0;
    let r = 3.5 * mm;
    let along_top: Vec<f32> = points.iter().filter(|p| p.1 == 0.0).map(|p| p.0).collect();
    let (left, right) = (
        along_top.iter().copied().fold(f32::MAX, f32::min),
        along_top.iter().copied().fold(f32::MIN, f32::max),
    );
    assert!(
        left == r && right == w - r,
        "the top runs straight only from {left} to {right}, not from corner to corner"
    );
    let side = points
        .iter()
        .filter(|p| p.0 == w)
        .map(|p| p.1)
        .fold(f32::MAX, f32::min);
    assert_eq!(side, r, "the side straightens {} mm down", side / mm);
    let corner: Vec<(f32, f32)> = points
        .iter()
        .copied()
        .filter(|p| p.0 >= w - r && p.1 <= r)
        .collect();
    assert!(
        (3..=5).contains(&corner.len()),
        "the corner is cut in {} points",
        corner.len()
    );
    for p in corner {
        let d = ((p.0 - (w - r)).powi(2) + (p.1 - r).powi(2)).sqrt();
        assert!(
            (d - r).abs() < 0.05,
            "the corner at {p:?} is {} mm from its middle, not 3.5",
            d / mm
        );
    }
}

fn playful(n: usize) -> Shelf {
    let mut s = faced(n);
    s.set_playful(true);
    s
}

#[test]
fn a_cart_left_alone_does_more_than_sway() {
    let mut s = playful(3);
    run(&mut s, 2.0);
    let (mut yaw, mut lift, mut roll) = (0f32, 0f32, 0f32);
    for _ in 0..120 * 60 {
        s.update(DT);
        let p = s.pose();
        yaw = yaw.max(p.yaw.abs());
        lift = lift.max(p.lift);
        roll = roll.max(p.roll.abs());
    }
    assert!(yaw > 30f32.to_radians(), "no turn beyond the sway: {yaw}");
    assert!(lift > 6.0, "no hop above the float: {lift}");
    assert!(roll > 0.08, "no roll in the canvas: {roll}");
}

#[test]
fn a_plain_shelf_keeps_to_the_sway() {
    let mut s = faced(3);
    run(&mut s, 2.0);
    for _ in 0..120 * 60 {
        s.update(DT);
        let p = s.pose();
        assert!(p.yaw.abs() <= 20.1f32.to_radians() && p.lift <= 4.01 && p.roll == 0.0);
    }
}

#[test]
fn a_scroll_ends_a_trick_and_the_cart_bounces_as_it_lands() {
    let mut s = playful(5);
    run(&mut s, 3.4);
    s.right();
    run(&mut s, 0.15);
    let p = s.pose();
    assert_eq!(
        (p.lift, p.roll),
        (0.0, 0.0),
        "a trick went on through the scroll"
    );
    let mut best = 0f32;
    for _ in 0..90 {
        s.update(DT);
        best = best.max(s.pose().lift);
    }
    assert!(best > 2.0, "the cart landed without a bounce: {best}");
}

#[test]
fn the_others_float_a_little_out_of_step() {
    let mut s = playful(5);
    run(&mut s, 3.0);
    let side_ys = |s: &Shelf| -> Vec<f32> {
        row(s)
            .iter()
            .filter_map(|d| match *d {
                Draw::Tex { y, tex, .. } | Draw::Spun { y, tex, .. }
                    if tex != TexId::from_raw(40 + s.index) =>
                {
                    Some(y)
                }
                _ => None,
            })
            .collect()
    };
    let before = side_ys(&s);
    run(&mut s, 0.7);
    assert!(
        before.iter().zip(side_ys(&s)).any(|(a, b)| *a != b),
        "the neighbours stand dead still"
    );
}

#[test]
fn every_model_has_a_back_as_big_as_its_face_outline() {
    use cartridge_ui::{cart_mesh, outline};
    let area = |v: &[cartridge_gfx::MeshVertex], z: f32, floor: f32| -> f32 {
        v.chunks(3)
            .filter(|t| t.iter().all(|p| p.normal[2] == z && p.xyz[2] <= floor))
            .map(|t| {
                let (a, b, c) = (t[0].xyz, t[1].xyz, t[2].xyz);
                ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
            })
            .sum()
    };
    for model in CartModel::ALL {
        let mesh = cart_mesh(model);
        // the plain back, not what is moulded on it
        let back = area(&mesh, 1.0, model.thick() + 1e-6);
        assert!(back > 0.0, "{model:?} has no back");
        if model == CartModel::Disc {
            continue;
        }
        let (points, (uw, uh)) = outline(model);
        let area = |points: &[(f32, f32)]| -> f32 {
            (0..points.len())
                .map(|i| {
                    let (a, b) = (points[i], points[(i + 1) % points.len()]);
                    a.0 * b.1 - b.0 * a.1
                })
                .sum::<f32>()
                .abs()
                / 2.0
        };
        // the back has its holes through it, which are no part of it
        let shoelace =
            (area(&points) - holes(model).iter().map(|h| area(h)).sum::<f32>()) / (uw * uh);
        assert!(
            (back - shoelace).abs() < 0.002,
            "{model:?}: the back covers {back} of the drawing, its outline {shoelace}"
        );
    }
}
