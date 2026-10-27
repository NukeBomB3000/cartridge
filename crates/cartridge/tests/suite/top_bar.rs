//! The top bar's Wi-Fi and ssh follow the radio on their own. A network the radio joins after
//! the handheld has booted, or one it loses, lights them or puts them out within a glance, on
//! the shelf, with no button pressed and no menu opened on the way.

use crate::common;

use std::sync::{Arc, Mutex};

use cartridge::app::{App, Phase};
use cartridge::wifi::{Saved, Status, WifiJob, WifiJobs, WifiNews, GLANCE_MS};
use cartridge_input::Millis;
use cartridge_store::{write_app_state, AppState};
use cartridge_ui::{Draw, LinkFaces, TexId};
use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use tempfile::TempDir;

/// Each link's faces, off then on.
const WIFI: [TexId; 2] = [TexId::from_raw(900), TexId::from_raw(901)];
const BLUETOOTH: [TexId; 2] = [TexId::from_raw(902), TexId::from_raw(903)];
const SSH: [TexId; 2] = [TexId::from_raw(904), TexId::from_raw(905)];

const FRAME_MS: Millis = 16;

/// The longest the bar may lag the radio: a glance, and the frames that take its news and the
/// status it asks for.
const LAG_MS: Millis = GLANCE_MS + 4 * FRAME_MS;

#[derive(Default)]
struct AirState {
    joined: bool,
    asked: Vec<WifiJob>,
    news: Vec<WifiNews>,
}

/// A radio that answers each job as the card's script would, from whether it has joined Home
/// yet, which the test changes behind the app's back as wpa_supplicant does.
#[derive(Clone, Default)]
struct Air(Arc<Mutex<AirState>>);

impl Air {
    fn set_joined(&self, joined: bool) {
        self.0.lock().unwrap().joined = joined;
    }

    fn asked(&self) -> Vec<WifiJob> {
        self.0.lock().unwrap().asked.clone()
    }
}

impl WifiJobs for Air {
    fn present(&self) -> bool {
        true
    }

    fn ask(&mut self, job: WifiJob) {
        let mut air = self.0.lock().unwrap();
        air.asked.push(job.clone());
        let joined = air.joined;
        let status = || {
            WifiNews::Status(Status {
                enabled: true,
                state: if joined { "COMPLETED" } else { "SCANNING" }.into(),
                ssid: joined.then(|| b"Home".to_vec()),
                ip: joined.then(|| "192.168.1.20".into()),
                rssi: joined.then_some(-48),
                mac: None,
                hostname: "rg35xxsp".into(),
                ntp: true,
                ntp_synced: joined,
            })
        };
        let saved = || {
            WifiNews::Saved(vec![Saved {
                id: 0,
                ssid: b"Home".to_vec(),
                current: joined,
                refused: false,
            }])
        };
        let news = match job {
            WifiJob::Up => vec![WifiNews::Started(true), status(), saved()],
            WifiJob::Refresh => vec![status(), saved()],
            WifiJob::Glance => vec![WifiNews::Glanced(joined)],
            _ => Vec::new(),
        };
        air.news.extend(news);
    }

    fn news(&mut self) -> Vec<WifiNews> {
        std::mem::take(&mut self.0.lock().unwrap().news)
    }
}

struct Rig {
    _d: TempDir,
    a: App,
    air: Air,
    now: Millis,
}

impl Rig {
    /// The shelf just booted with Wi-Fi left on and Home saved, which the radio has `joined`
    /// already or not yet.
    fn booted(joined: bool) -> Rig {
        let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
        let state = AppState {
            clock_set: true,
            wifi: true,
            ..AppState::default()
        };
        write_app_state(d.path(), &state).expect("write cartridge.state");
        let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
        let air = Air::default();
        air.set_joined(joined);
        a.set_wifi_jobs(Box::new(air.clone()));
        a.set_link_faces(LinkFaces {
            wifi: WIFI.map(|t| (t, 16, 16)),
            bluetooth: BLUETOOTH.map(|t| (t, 16, 16)),
            ssh: SSH.map(|t| (t, 24, 16)),
        });
        assert!(matches!(a.phase(), Phase::Shelf), "{:?}", a.phase());
        Rig {
            _d: d,
            a,
            air,
            now: 1_000,
        }
    }

    /// Runs the shelf `ms` on, a frame at a time, with no button pressed.
    fn frames(&mut self, ms: Millis) {
        for _ in 0..ms / FRAME_MS {
            self.now += FRAME_MS;
            self.a.tick_ms(self.now);
        }
    }

    /// Whether the top bar draws `pair`'s face that is on, or None when it draws neither.
    fn lit(&self, pair: [TexId; 2]) -> Option<bool> {
        let mut out = Vec::new();
        self.a.draw(&mut out);
        out.iter().find_map(|d| match d {
            Draw::Tex { tex, .. } if pair.contains(tex) => Some(*tex == pair[1]),
            _ => None,
        })
    }

    /// How long the shelf runs before the top bar draws `pair` `on`, if it does within `limit`.
    fn until(&mut self, pair: [TexId; 2], on: bool, limit: Millis) -> Option<Millis> {
        let from = self.now;
        while self.now - from <= limit {
            if self.lit(pair) == Some(on) {
                return Some(self.now - from);
            }
            self.frames(FRAME_MS);
        }
        None
    }
}

#[test]
fn wifi_joined_after_boot_lights_the_top_bar_within_a_glance() {
    let mut r = Rig::booted(false);
    r.frames(3_000);
    assert_eq!(r.lit(WIFI), Some(false), "Wi-Fi lit with nothing joined");
    assert!(
        !r.air.asked().contains(&WifiJob::Refresh),
        "the shelf asked for the whole status while nothing changed: {:?}",
        r.air.asked()
    );
    r.air.set_joined(true);
    let took = r.until(WIFI, true, 5 * GLANCE_MS);
    assert!(
        took.is_some_and(|ms| ms <= LAG_MS),
        "Wi-Fi lit after {took:?} ms, not within {LAG_MS}"
    );
}

#[test]
fn ssh_lights_with_a_network_joined_after_boot_within_a_glance() {
    let mut r = Rig::booted(false);
    r.frames(3_000);
    assert_eq!(r.lit(SSH), Some(false), "ssh lit with nothing joined");
    r.air.set_joined(true);
    let took = r.until(SSH, true, 5 * GLANCE_MS);
    assert!(
        took.is_some_and(|ms| ms <= LAG_MS),
        "ssh lit after {took:?} ms, not within {LAG_MS}"
    );
}

#[test]
fn a_network_lost_puts_wifi_and_ssh_out_within_a_glance() {
    let mut r = Rig::booted(true);
    r.frames(FRAME_MS);
    assert_eq!(r.lit(WIFI), Some(true), "Wi-Fi is not lit once joined");
    assert_eq!(r.lit(SSH), Some(true), "ssh is not lit once joined");
    r.frames(3_000);
    r.air.set_joined(false);
    let took = r.until(WIFI, false, 5 * GLANCE_MS);
    assert!(
        took.is_some_and(|ms| ms <= LAG_MS),
        "Wi-Fi went out after {took:?} ms, not within {LAG_MS}"
    );
    assert_eq!(r.lit(SSH), Some(false), "ssh stayed lit with Wi-Fi out");
}
