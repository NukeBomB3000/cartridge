//! Network, opened from Settings: switching Wi-Fi, network time and SSH, Y on each for its
//! panel, and the networks in reach, looked for when X asks.

use cartridge_input::Action;
use cartridge_ui::kit::bar::Hint;
use cartridge_ui::network::NetworkFaces;
use cartridge_ui::{Draw, SettingsRow};

use crate::app::{App, Phase};
use crate::nav::Dir;
use crate::screens::bar::{FORGET, INFO, SCAN};
use crate::wifi_screen::{NetworkView, Outcome as WifiOutcome, Typing, WifiScreen};

impl App {
    pub(crate) fn network_input(&mut self, action: Action) {
        let now = self.now();
        let Phase::Network(screen) = &mut self.phase else {
            return;
        };
        let before = screen.clone();
        let outcome = screen.input(&mut self.wifi, self.state.ssh, action, now);
        // the passphrase's keyboard slides in over the list and away again
        match (before.typing.is_some(), screen.typing.is_some()) {
            (false, true) => self.slide_from(Phase::Network(before), Dir::Deeper),
            (true, false) => self.slide_from(Phase::Network(before), Dir::Back),
            _ => {}
        }
        match outcome {
            WifiOutcome::Close => self.go(
                Phase::Settings {
                    row: SettingsRow::Network,
                },
                Dir::Back,
            ),
            WifiOutcome::Info(topic) => self.open_info(topic),
            WifiOutcome::SwitchSsh(on) => self.set_ssh(on),
            WifiOutcome::Stay => {}
        }
        if self.state.wifi != self.wifi.enabled() {
            self.state.wifi = self.wifi.enabled();
            self.persist();
        }
    }

    /// What the Network modal shows, while it is open and not taking a passphrase.
    pub fn network_view(&mut self) -> Option<NetworkView> {
        match &mut self.phase {
            Phase::Network(screen) if screen.typing.is_none() => {
                Some(screen.view(&self.wifi, self.state.ssh))
            }
            _ => None,
        }
    }

    /// The Network screen's keyboard, and the network it is for.
    pub fn network_typing(&self) -> Option<&Typing> {
        match &self.phase {
            Phase::Network(screen) => screen.typing.as_ref(),
            _ => None,
        }
    }

    /// Whether Y forgets the network under the cursor, for the bar to offer Y FORGET: the
    /// Network modal is open on a network the handheld has saved.
    pub fn network_forgets(&self) -> bool {
        match &self.phase {
            Phase::Network(screen) if screen.typing.is_none() => screen.forgets(&self.wifi),
            _ => false,
        }
    }

    /// Network's keys beyond the bar's two actions while no passphrase is being typed: X SCAN
    /// while Wi-Fi is on, and beside it Y INFO on NETWORK's own rows or Y FORGET on a saved
    /// network (`network_forgets`' rule).
    pub(crate) fn network_hints(&self, screen: &WifiScreen) -> Vec<Hint<'static>> {
        if screen.typing.is_some() {
            return Vec::new();
        }
        let scan = self.wifi.enabled().then_some(SCAN);
        let y = if screen.informs() {
            Some(INFO)
        } else {
            screen.forgets(&self.wifi).then_some(FORGET)
        };
        scan.into_iter().chain(y).collect()
    }

    pub fn set_network_faces(&mut self, faces: NetworkFaces) {
        self.network_faces = Some(faces);
    }

    pub(crate) fn draw_network(&self, out: &mut Vec<Draw>) {
        if let Some(faces) = &self.network_faces {
            faces.draw(out);
        }
    }
}
