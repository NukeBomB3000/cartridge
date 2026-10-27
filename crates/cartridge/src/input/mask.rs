use cartridge_input::{Action, Btn, Keys};
use cartridge_retro::ButtonMask;

/// The buttons held for the game, as the core reads them.
#[derive(Default)]
pub struct Pad {
    mask: u16,
    /// L2 and R2 are the game's, as the PlayStation's are: only then do they reach its pad.
    shoulders: bool,
}

impl Pad {
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::GbaDown(b) => {
                if let Some(bit) = self.bit(b) {
                    self.mask |= bit;
                }
            }
            Action::GbaUp(b) => {
                if let Some(bit) = self.bit(b) {
                    self.mask &= !bit;
                }
            }
            _ => {}
        }
    }

    /// What the buttons do for the game being played: where L2 and R2 stop being the game's,
    /// any still held are let go.
    pub fn set_keys(&mut self, keys: Keys) {
        self.shoulders = keys.game_l2_r2;
        if !self.shoulders {
            self.mask &= !(ButtonMask::L2 | ButtonMask::R2);
        }
    }

    pub fn mask(&self) -> ButtonMask {
        ButtonMask(self.mask)
    }

    pub fn clear(&mut self) {
        self.mask = 0;
    }

    fn bit(&self, btn: Btn) -> Option<u16> {
        Some(match btn {
            Btn::Up => ButtonMask::UP,
            Btn::Down => ButtonMask::DOWN,
            Btn::Left => ButtonMask::LEFT,
            Btn::Right => ButtonMask::RIGHT,
            Btn::A => ButtonMask::A,
            Btn::B => ButtonMask::B,
            Btn::X => ButtonMask::X,
            Btn::Y => ButtonMask::Y,
            Btn::L1 => ButtonMask::L,
            Btn::R1 => ButtonMask::R,
            Btn::L2 if self.shoulders => ButtonMask::L2,
            Btn::R2 if self.shoulders => ButtonMask::R2,
            Btn::Start => ButtonMask::START,
            Btn::Select => ButtonMask::SELECT,
            _ => return None,
        })
    }
}
