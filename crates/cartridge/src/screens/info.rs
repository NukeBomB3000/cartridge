//! The panels Y opens from Network's NETWORK rows, each a lone panel of lines with a note under
//! them: WI-FI, the network joined and how the handheld is known on it; NETWORK TIME, whether
//! a time server has set the clock and what it says; and SSH, how to log in from a computer.
//! The rows themselves only switch each one on and off. B goes back to the row.
//!
//! While there is nothing to tell (Wi-Fi off, say) a panel has no lines, only a note saying
//! what to do first.

use cartridge_input::{Action, Btn};
use cartridge_store::civil_from_days;
use cartridge_ui::network::InfoFaces;
use cartridge_ui::{hhmm, zone_text, Draw};

use crate::app::{App, Phase};
use crate::nav::Dir;
use crate::screens::ssh::SshScreen;
use crate::ssh::read_ssh_password;
use crate::wifi::{shown, Status};
use crate::wifi_screen::{Topic, Wifi, WifiScreen};

/// A panel Y opened from NETWORK.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InfoScreen {
    Wifi,
    Time,
    Ssh(SshScreen),
}

impl InfoScreen {
    pub fn topic(&self) -> Topic {
        match self {
            InfoScreen::Wifi => Topic::Wifi,
            InfoScreen::Time => Topic::Time,
            InfoScreen::Ssh(_) => Topic::Ssh,
        }
    }
}

/// What a panel shows: its title, each line a name and its value, then a note under them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfoView {
    pub title: &'static str,
    pub lines: Vec<(&'static str, String)>,
    pub note: &'static str,
}

impl InfoView {
    /// A panel with nothing to tell but `note`.
    pub fn note(title: &'static str, note: &'static str) -> InfoView {
        InfoView {
            title,
            lines: Vec::new(),
            note,
        }
    }
}

/// The most of a network's name WI-FI's NETWORK line holds: what is left of the panel's row
/// beside the word NETWORK.
pub const NETWORK_SHOWN: usize = 22;

/// A network's name as WI-FI's panel shows it, cut short with an ellipsis when it is long.
fn network_name(ssid: &[u8]) -> String {
    let name = shown(ssid);
    if name.chars().count() <= NETWORK_SHOWN {
        return name;
    }
    let cut: String = name.chars().take(NETWORK_SHOWN - 1).collect();
    format!("{}…", cut.trim_end())
}

/// The name the handheld answers to on the network: `<hostname>.local`, or its address when
/// it has no name.
pub fn host(status: &Status, ip: &str) -> String {
    match status.hostname.as_str() {
        "" => ip.to_string(),
        name => format!("{name}.local"),
    }
}

/// WI-FI: the network joined, its signal, the handheld's address and name on it, and its
/// radio's hardware address.
pub fn wifi_view(wifi: &Wifi) -> InfoView {
    const TITLE: &str = "WI-FI";
    let status = wifi.status();
    if !wifi.present() {
        return InfoView::note(TITLE, "This handheld has no Wi-Fi.");
    }
    if !wifi.enabled() {
        return InfoView::note(TITLE, "Turn Wi-Fi on first.");
    }
    let Some(ssid) = status.ssid.as_ref().filter(|_| status.joined()) else {
        return InfoView::note(TITLE, "Not connected to a network.");
    };
    let Some(ip) = &status.ip else {
        return InfoView::note(TITLE, "Waiting for an address…");
    };
    let mut lines = vec![("NETWORK", network_name(ssid))];
    if let Some(rssi) = status.rssi {
        lines.push(("SIGNAL", format!("{rssi} dBm")));
    }
    lines.push(("IP", ip.clone()));
    lines.push(("HOST", host(status, ip)));
    if let Some(mac) = &status.mac {
        lines.push(("MAC", mac.clone()));
    }
    InfoView {
        title: TITLE,
        lines,
        note: "Networks: Config/wifi.conf",
    }
}

/// NETWORK TIME: whether a time server has set the clock, and the date, time and zone it shows,
/// the clock being `wall` seconds (local) in a zone `offset_min` minutes from UTC.
pub fn time_view(wifi: &Wifi, wall: i64, offset_min: i16) -> InfoView {
    const TITLE: &str = "NETWORK TIME";
    let status = wifi.status();
    if !status.ntp {
        return InfoView::note(TITLE, "Turn Network Time on first.");
    }
    let (year, month, day) = civil_from_days(wall.div_euclid(86_400));
    let synced = status.ntp_synced;
    let lines = vec![
        (
            "STATUS",
            if synced { "SYNCED" } else { "WAITING" }.to_string(),
        ),
        ("DATE", format!("{year:04}-{month:02}-{day:02}")),
        ("TIME", hhmm(wall)),
        ("ZONE", zone_text(i64::from(offset_min))),
    ];
    let note = if synced {
        "Set the zone in Date & Time"
    } else if wifi.enabled() && status.joined() {
        "Waiting for a time server…"
    } else {
        "Join a Wi-Fi network first."
    };
    InfoView {
        title: TITLE,
        lines,
        note,
    }
}

impl App {
    /// Opens the panel Y asked for on NETWORK's row of `topic`.
    pub(crate) fn open_info(&mut self, topic: Topic) {
        let screen = match topic {
            Topic::Wifi => InfoScreen::Wifi,
            Topic::Time => InfoScreen::Time,
            Topic::Ssh => InfoScreen::Ssh(SshScreen {
                password: read_ssh_password(self.root.as_deref()),
                on: self.state.ssh,
                trouble: self.ssh.trouble(),
            }),
        };
        self.go(Phase::Info(screen), Dir::Deeper);
    }

    pub(crate) fn info_input(&mut self, action: Action) {
        let Phase::Info(screen) = &self.phase else {
            return;
        };
        if matches!(action, Action::GbaDown(Btn::B) | Action::QuickMenu) {
            let back = Phase::Network(WifiScreen {
                sel: screen.topic().row(),
                ..WifiScreen::default()
            });
            self.go(back, Dir::Back);
        }
    }

    /// The open panel's lines, while one is open.
    pub fn info_view(&self) -> Option<InfoView> {
        let Phase::Info(screen) = &self.phase else {
            return None;
        };
        Some(match screen {
            InfoScreen::Wifi => wifi_view(&self.wifi),
            InfoScreen::Time => time_view(&self.wifi, self.wall_secs(), self.state.utc_offset_min),
            InfoScreen::Ssh(ssh) => ssh.view(self.wifi.status()),
        })
    }

    pub fn set_info_faces(&mut self, faces: InfoFaces) {
        self.info_faces = Some(faces);
    }

    pub(crate) fn draw_info(&self, out: &mut Vec<Draw>) {
        if let Some(faces) = &self.info_faces {
            faces.draw(out);
        }
    }
}
