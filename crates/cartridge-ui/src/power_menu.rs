//! The two things holding POWER offers. The menu and the screen after its choice are lone kit
//! panels, drawn by cartridge's screens/power.rs.

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PowerChoice {
    Restart,
    PowerOff,
}

impl PowerChoice {
    pub const ALL: [PowerChoice; 2] = [PowerChoice::Restart, PowerChoice::PowerOff];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            PowerChoice::Restart => "Restart",
            PowerChoice::PowerOff => "Power Off",
        }
    }
}
