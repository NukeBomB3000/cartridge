use cartridge::app::{GameMenu, LinkRow};
use cartridge::link_kind::LinkKind;
use cartridge::link_screen::*;
use cartridge::link_start::{LinkFail, LinkStep};
use cartridge_ui::kit::fade::stepped;
use cartridge_ui::link_art::{ADAPTER_H, ADAPTER_W, CONNECTOR_H, PLUG_H, TIP_H};
use cartridge_ui::{Draw, TexId, SPRITE_PX};

fn working(role: LinkRow, since: u64) -> GameMenu {
    GameMenu::Working {
        role,
        step: LinkStep::Waiting,
        since,
    }
}

fn failed(since: u64) -> GameMenu {
    GameMenu::Failed {
        role: LinkRow::Host,
        fail: LinkFail::NobodyCame,
        worked: 0,
        since,
    }
}

fn linked(worked: u64, since: u64) -> GameMenu {
    GameMenu::Linked {
        role: LinkRow::Host,
        worked,
        since,
        opened: false,
    }
}

const SEATED: i32 = TIP_H as i32;

#[test]
fn pick_hangs_the_plug_over_the_port() {
    let at = plug_bottom(GameMenu::Pick(LinkRow::Host), 0);
    assert!(at < 0, "the plug's tip is in the port before A: {at}");
}

#[test]
fn working_seats_the_plug_in_200ms_and_holds_it_there() {
    let m = working(LinkRow::Host, 1000);
    assert_eq!(
        plug_bottom(m, 1000),
        plug_bottom(GameMenu::Pick(LinkRow::Host), 0)
    );
    assert_eq!(
        plug_bottom(m, 1200),
        SEATED,
        "the tip is not all the way in"
    );
    let held: Vec<i32> = (0..3000)
        .step_by(50)
        .map(|t| plug_bottom(m, 1200 + t))
        .collect();
    assert!(held.iter().all(|y| *y == SEATED), "{held:?}");
}

#[test]
fn linked_seats_the_plug_from_where_it_was_and_clicks_as_it_lands() {
    let (worked, since) = (1000, 1100);
    let before = plug_bottom(working(LinkRow::Host, worked), since);
    let m = linked(worked, since);
    assert_eq!(plug_bottom(m, since), before, "the plug jumped");
    assert_eq!(plug_bottom(m, since + 160), SEATED);
    assert!(!clicks_shown(m, since + 100));
    assert!(clicks_shown(m, since + 160));
}

#[test]
fn a_failed_plug_lifts_out_higher_than_it_was_picked_and_shakes_a_pixel_each_way() {
    let m = failed(5000);
    let pick = plug_bottom(GameMenu::Pick(LinkRow::Host), 0);
    assert!(plug_bottom(m, 5250) < pick, "the plug did not lift out");
    assert_eq!(shake(m, 5000), 0, "it shook before it lifted");
    let shook: Vec<i32> = (5250..5450).step_by(10).map(|t| shake(m, t)).collect();
    assert!(shook.contains(&1) && shook.contains(&-1), "{shook:?}");
    assert!(shook.iter().all(|dx| dx.abs() <= 1), "{shook:?}");
    assert_eq!(shake(m, 6000), 0, "it never came to rest");
    assert_eq!(shake(working(LinkRow::Host, 0), 300), 0);
}

#[test]
fn the_adapter_seats_while_working_and_stays_seated() {
    assert!(adapter_bottom(GameMenu::Pick(LinkRow::Join), 0) < 0);
    assert_eq!(adapter_bottom(working(LinkRow::Join, 0), 200), 0);
    assert_eq!(adapter_bottom(failed(3000), 3250), 0);
}

#[test]
fn the_arcs_call_in_turn_hold_when_linked_and_die_on_failure() {
    let a = arc_alphas(working(LinkRow::Host, 0), 200 + 600);
    assert_eq!(a[0], 1.0, "ring 0 is not at its peak: {a:?}");
    assert!(a[1] < 1.0);
    assert_eq!(arc_alphas(GameMenu::Pick(LinkRow::Host), 0), [0.0; 3]);
    assert_eq!(arc_alphas(linked(0, 4000), 4160), [1.0; 3]);
    assert_eq!(arc_alphas(failed(4000), 4250), [0.0; 3]);
}

#[test]
fn the_arcs_fade_in_the_four_steps_of_a_kit_fade() {
    let menus = [
        working(LinkRow::Host, 0),
        linked(0, 900),
        failed(900),
        GameMenu::Unplug {
            role: LinkRow::Host,
            since: 900,
        },
    ];
    for m in menus {
        for t in (0..3000).step_by(7) {
            for a in arc_alphas(m, t) {
                assert_eq!(a, stepped(a), "{m:?} at {t}: {a} is between steps");
            }
        }
    }
}

fn sprites() -> LinkSprites {
    let s = |n: usize, w: u32, h: u32| Sprite {
        tex: TexId::from_raw(n),
        w,
        h,
    };
    LinkSprites {
        plug_host: s(1, 30, PLUG_H),
        plug_join: s(2, 36, PLUG_H),
        cable: s(3, 6, 1),
        adapter: s(4, ADAPTER_W, ADAPTER_H),
        port: s(5, 320, PORT_UP),
        arcs_right: [s(6, 5, 14), s(7, 7, 22), s(8, 9, 30)],
        arcs_left: [s(9, 5, 14), s(10, 7, 22), s(11, 9, 30)],
        clicks: s(12, 52, 10),
    }
}

fn drawn(menu: GameMenu, kind: LinkKind, now: u64) -> (LinkSprites, Vec<Draw>) {
    let s = sprites();
    let mut out = Vec::new();
    draw_link_art(menu, kind, now, &s, 8, &mut out);
    (s, out)
}

fn find(out: &[Draw], tex: TexId) -> Option<(f32, f32, f32, f32)> {
    out.iter().find_map(|d| match *d {
        Draw::Tex {
            x, y, w, h, tex: t, ..
        } if t == tex => Some((x, y, w, h)),
        _ => None,
    })
}

fn texes(out: &[Draw]) -> Vec<TexId> {
    out.iter()
        .filter_map(|d| match d {
            Draw::Tex { tex, .. } | Draw::Turned { tex, .. } => Some(*tex),
            _ => None,
        })
        .collect()
}

const PX: f32 = SPRITE_PX as f32;

#[test]
fn a_searching_plug_sits_in_the_port_under_the_strip() {
    let (s, out) = drawn(working(LinkRow::Host, 0), LinkKind::Cable, 1000);
    let (_, y, _, h) = find(&out, s.plug_host.tex).expect("no plug");
    let bottom = (y + h) / PX;
    assert_eq!(
        bottom as i32,
        port_top() + SEATED,
        "the plug is not seated in the port"
    );
    let t = texes(&out);
    let at = |tex| t.iter().position(|x| *x == tex);
    assert!(
        at(s.port.tex) > at(s.plug_host.tex),
        "the port must be drawn over the plug"
    );
}

#[test]
fn the_cable_runs_from_the_panel_down_to_the_plugs_neck() {
    let (s, out) = drawn(GameMenu::Pick(LinkRow::Host), LinkKind::Cable, 0);
    let (cx, cy, cw, ch) = find(&out, s.cable.tex).expect("no cable");
    let (px, py, pw, _) = find(&out, s.plug_host.tex).expect("no plug");
    assert_eq!(
        cy,
        8.0 * PX,
        "the cable does not start where it was asked to"
    );
    assert_eq!(cy + ch, py, "the cable does not reach the plug");
    assert_eq!(
        cx + cw / 2.0,
        px + pw / 2.0,
        "the cable is not over the neck"
    );
}

#[test]
fn cable_carts_draw_the_plug_and_wireless_carts_the_adapter() {
    let (s, out) = drawn(GameMenu::Pick(LinkRow::Host), LinkKind::Cable, 0);
    let t = texes(&out);
    assert!(t.contains(&s.plug_host.tex) && t.contains(&s.port.tex));
    assert!(!t.contains(&s.adapter.tex));

    let (s, out) = drawn(GameMenu::Pick(LinkRow::Host), LinkKind::Wireless, 0);
    let t = texes(&out);
    assert!(t.contains(&s.adapter.tex) && !t.contains(&s.plug_host.tex));
    assert!(!t.contains(&s.cable.tex), "a cable on the adapter");
}

#[test]
fn a_joiner_holds_the_grey_plug() {
    let (s, out) = drawn(GameMenu::Pick(LinkRow::Join), LinkKind::Cable, 0);
    let t = texes(&out);
    assert!(t.contains(&s.plug_join.tex) && !t.contains(&s.plug_host.tex));
}

#[test]
fn nothing_is_turned_or_off_the_art_grid() {
    let menus = [
        GameMenu::Pick(LinkRow::Host),
        working(LinkRow::Join, 0),
        linked(0, 300),
        failed(300),
        GameMenu::Unplug {
            role: LinkRow::Host,
            since: 300,
        },
    ];
    for m in menus {
        for kind in [LinkKind::Cable, LinkKind::Wireless] {
            for t in (0..1500).step_by(13) {
                let (_, out) = drawn(m, kind, t);
                for d in &out {
                    let Draw::Tex { x, y, w, h, .. } = *d else {
                        panic!("{m:?} drew {d:?}, which is not a face on the grid");
                    };
                    for v in [x, y, w, h] {
                        assert_eq!(v % PX, 0.0, "{m:?} at {t}: {d:?} is off the art grid");
                    }
                }
            }
        }
    }
}

#[test]
fn unplug_pulls_the_plug_back_out_of_the_port_and_leaves_it_there() {
    let m = GameMenu::Unplug {
        role: LinkRow::Host,
        since: 1000,
    };
    let pick = plug_bottom(GameMenu::Pick(LinkRow::Host), 0);
    assert_eq!(
        plug_bottom(m, 1000),
        SEATED,
        "the unplug did not start seated"
    );
    assert_eq!(
        plug_bottom(m, 1260),
        pick,
        "the plug is not back where it was picked up"
    );
    let held: Vec<i32> = (0..2000)
        .step_by(100)
        .map(|t| plug_bottom(m, 1260 + t))
        .collect();
    assert!(held.iter().all(|y| *y == pick), "{held:?}");
}

#[test]
fn unplug_lifts_the_adapter_off_the_port_and_its_arcs_die() {
    let m = GameMenu::Unplug {
        role: LinkRow::Join,
        since: 0,
    };
    assert_eq!(adapter_bottom(m, 0), 0);
    assert_eq!(
        adapter_bottom(m, 260),
        adapter_bottom(GameMenu::Pick(LinkRow::Join), 0)
    );
    assert_eq!(
        arc_alphas(m, 0),
        [1.0; 3],
        "a seated adapter's arcs do not start full"
    );
    assert_eq!(arc_alphas(m, 260), [0.0; 3], "the arcs outlived the link");
    let mid = arc_alphas(m, 130);
    assert!(
        mid[0] > 0.0 && mid[0] < 1.0,
        "the arcs did not fade: {mid:?}"
    );
}

#[test]
fn the_adapter_calls_out_with_its_arcs_either_side_while_working() {
    let (s, out) = drawn(working(LinkRow::Host, 0), LinkKind::Wireless, 800);
    let (ax, ay, aw, _) = find(&out, s.adapter.tex).expect("no adapter");
    let (rx, ry, _, rh) = find(&out, s.arcs_right[0].tex).expect("no right arc");
    let (lx, _, lw, _) = find(&out, s.arcs_left[0].tex).expect("no left arc");
    assert!(
        rx >= ax + aw && lx + lw <= ax,
        "the arcs are not either side of it"
    );
    let body = ((ADAPTER_H - CONNECTOR_H) as f32) * PX;
    assert_eq!(
        ry + rh / 2.0,
        ay + body / 2.0,
        "the arcs are not level with its body"
    );
}
