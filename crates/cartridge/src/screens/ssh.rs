//! SSH: switched on and off by Network's SSH row, and its panel, opened by Y on that row: how
//! to log in to the handheld from a computer on the same network. SFTP comes with it.

use crate::app::App;
use crate::screens::info::{host, InfoView};
use crate::ssh::{Ssh, SshServer, Trouble, SSH_PORT};
use crate::wifi::Status;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshScreen {
    /// root's password, read from baseos.conf when the screen opened.
    pub password: String,
    /// SSH was on when the screen opened.
    pub on: bool,
    /// What went wrong giving root a password of its own, when something did.
    pub trouble: Option<Trouble>,
}

impl SshScreen {
    /// How to log in, as the radio's `status` says the handheld can be reached.
    pub fn view(&self, status: &Status) -> InfoView {
        const TITLE: &str = "SSH";
        if !self.on {
            return InfoView::note(TITLE, "Turn SSH on first.");
        }
        if !status.joined() {
            return InfoView::note(TITLE, "Join a Wi-Fi network first.");
        }
        let Some(ip) = &status.ip else {
            return InfoView::note(TITLE, "Waiting for an address…");
        };
        InfoView {
            title: TITLE,
            lines: vec![
                ("IP", ip.clone()),
                ("HOST", host(status, ip)),
                ("USER", "root".to_string()),
                ("PASSWORD", self.password.clone()),
                ("PORT", SSH_PORT.to_string()),
            ],
            note: self
                .trouble
                .map_or("SFTP uses the same login", Trouble::note),
        }
    }
}

impl App {
    /// SSH is switched on.
    pub fn ssh_on(&self) -> bool {
        self.state.ssh
    }

    /// Switches SSH on or off: the server starts or stops, and the card remembers.
    pub(crate) fn set_ssh(&mut self, on: bool) {
        self.ssh.switch(on);
        if self.state.ssh != on {
            self.state.ssh = on;
            self.persist();
        }
    }

    /// Puts `server` in place of the system's, and does with it what cartridge does as it
    /// starts.
    pub fn set_ssh_server(&mut self, server: Box<dyn SshServer>) {
        self.ssh = Ssh::new(server);
        self.ssh.start(self.state.ssh, self.root.as_deref());
    }
}
