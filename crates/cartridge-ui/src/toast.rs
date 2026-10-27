//! What the HUD says just happened, each under the title of what it is about, drawn as the
//! kit's toast (kit::toast).

use crate::kit::palette::Palette;
use crate::kit::toast::toast_face;
use crate::plate::UndoFace;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Toast {
    StateSaved,
    StateLoaded,
    NeedsGpsp,
    NoLink,
    LinkEnded,
    PeerEnded,
    BiosMismatch,
    ColourOn,
    ColourOff,
    /// SELECT+B switched the LCD's ghosting on, or off.
    GhostingOn,
    GhostingOff,
    /// SELECT+Y on a television's console: the picture at its actual size, a whole scale.
    PictureActual,
    /// SELECT+Y on a television's console: the picture a television's 4:3.
    PictureAspect,
    /// SELECT+Y on a television's console: the picture stretched over the screen.
    PictureStretch,
    /// SELECT+Y on a handheld: the picture as it was always drawn.
    PictureOriginal,
    /// SELECT+Y on a handheld: the picture its own shape, as large as the screen holds.
    PictureFit,
    /// SELECT+X in a game: the screen look now chosen for its system.
    Shader(cartridge_gfx::Look),
    /// SELECT + MENU on a PlayStation game on one disc: there is no other to swap in.
    OneDisc,
    /// The DISC screen swapped disc `n`, counting from 1, into the drive.
    Disc(u8),
    /// The card is mounted read-only, so nothing saves: SELECT+START repairs it.
    CardReadOnly,
    /// The repair before this boot left the card sound, or did not.
    CardRepaired,
    CardRepairFailed,
    /// SELECT+START on a read-only card with too little charge for the repair.
    CardRepairLowBattery,
    /// SELECT+L2/R2 in a Game Boy game: the palette now chosen, and whether the game took it at
    /// once or takes it at its next start.
    Palette(cartridge_store::GbColors, bool),
    /// Picking Five Games' games: a sixth asked for, or START with none.
    PickAtMost,
    PickAtLeast,
    /// A short MENU press in a game with no guide in GUIDES for it.
    NoGuide,
}

/// What the toast says once disc `n` is in, for every disc a PlayStation game can have.
const DISC_IN: [&str; 9] = [
    "Disc 1 inserted",
    "Disc 2 inserted",
    "Disc 3 inserted",
    "Disc 4 inserted",
    "Disc 5 inserted",
    "Disc 6 inserted",
    "Disc 7 inserted",
    "Disc 8 inserted",
    "Disc 9 inserted",
];

impl Toast {
    /// Every toast there is, a disc's once, as disc 1, and a shader's once, as none.
    pub const ALL: [Toast; 27] = [
        Toast::StateSaved,
        Toast::StateLoaded,
        Toast::NeedsGpsp,
        Toast::NoLink,
        Toast::LinkEnded,
        Toast::PeerEnded,
        Toast::BiosMismatch,
        Toast::ColourOn,
        Toast::ColourOff,
        Toast::GhostingOn,
        Toast::GhostingOff,
        Toast::PictureActual,
        Toast::PictureAspect,
        Toast::PictureStretch,
        Toast::PictureOriginal,
        Toast::PictureFit,
        Toast::Shader(cartridge_gfx::Look::Off),
        Toast::OneDisc,
        Toast::Disc(1),
        Toast::CardReadOnly,
        Toast::CardRepaired,
        Toast::CardRepairFailed,
        Toast::CardRepairLowBattery,
        Toast::Palette(cartridge_store::GbColors::Auto, false),
        Toast::PickAtMost,
        Toast::PickAtLeast,
        Toast::NoGuide,
    ];

    /// Its place in `ALL`: every disc's toast is the one place.
    pub fn index(self) -> usize {
        match self {
            Toast::StateSaved => 0,
            Toast::StateLoaded => 1,
            Toast::NeedsGpsp => 2,
            Toast::NoLink => 3,
            Toast::LinkEnded => 4,
            Toast::PeerEnded => 5,
            Toast::BiosMismatch => 6,
            Toast::ColourOn => 7,
            Toast::ColourOff => 8,
            Toast::GhostingOn => 9,
            Toast::GhostingOff => 10,
            Toast::PictureActual => 11,
            Toast::PictureAspect => 12,
            Toast::PictureStretch => 13,
            Toast::PictureOriginal => 14,
            Toast::PictureFit => 15,
            Toast::Shader(..) => 16,
            Toast::OneDisc => 17,
            Toast::Disc(_) => 18,
            Toast::CardReadOnly => 19,
            Toast::CardRepaired => 20,
            Toast::CardRepairFailed => 21,
            Toast::CardRepairLowBattery => 22,
            Toast::Palette(..) => 23,
            Toast::PickAtMost => 24,
            Toast::PickAtLeast => 25,
            Toast::NoGuide => 26,
        }
    }

    /// What it is about, in the toast's title tab.
    pub fn label(self) -> &'static str {
        match self {
            Toast::StateSaved | Toast::StateLoaded => "SAVE STATE",
            Toast::NeedsGpsp
            | Toast::NoLink
            | Toast::LinkEnded
            | Toast::PeerEnded
            | Toast::BiosMismatch => "LINK",
            Toast::ColourOn | Toast::ColourOff => "COLOUR",
            Toast::GhostingOn | Toast::GhostingOff => "GHOSTING",
            Toast::PictureActual
            | Toast::PictureAspect
            | Toast::PictureStretch
            | Toast::PictureOriginal
            | Toast::PictureFit => "PICTURE",
            Toast::Shader(..) => "SHADER",
            Toast::OneDisc | Toast::Disc(_) => "DISC",
            Toast::CardReadOnly
            | Toast::CardRepaired
            | Toast::CardRepairFailed
            | Toast::CardRepairLowBattery => "SD CARD",
            Toast::Palette(_, true) => "PALETTE",
            Toast::Palette(_, false) => "PALETTE NEXT START",
            Toast::PickAtMost | Toast::PickAtLeast => "5 GAMES",
            Toast::NoGuide => "GUIDE",
        }
    }

    /// The line in the toast's well, in its own case.
    pub fn text(self) -> &'static str {
        match self {
            Toast::StateSaved => "State saved",
            Toast::StateLoaded => "State loaded",
            Toast::NeedsGpsp => "Please switch to gpSP",
            Toast::NoLink => "No link support",
            Toast::LinkEnded => "Link ended",
            Toast::PeerEnded => "Link was ended",
            Toast::BiosMismatch => "BIOS does not match",
            Toast::ColourOn => "Correction on",
            Toast::ColourOff => "Correction off",
            Toast::GhostingOn => "Ghosting on",
            Toast::GhostingOff => "Ghosting off",
            Toast::PictureActual => "Integer scale",
            Toast::PictureAspect => "Keep aspect 4:3",
            Toast::PictureStretch => "Stretch",
            Toast::PictureOriginal => "Original",
            Toast::PictureFit => "Keep aspect",
            Toast::Shader(look) => match look {
                cartridge_gfx::Look::Off => "Off",
                cartridge_gfx::Look::Lcd => "LCD",
                cartridge_gfx::Look::Grid => "Grid",
                cartridge_gfx::Look::Dot => "Dot",
                cartridge_gfx::Look::Simpletex => "Simpletex",
                cartridge_gfx::Look::Transparent => "Transparent",
                cartridge_gfx::Look::Dmg => "DMG",
                cartridge_gfx::Look::Scanlines => "Scanlines",
                cartridge_gfx::Look::Crt => "CRT",
                cartridge_gfx::Look::Curved => "CRT curved",
            },
            Toast::OneDisc => "Only one disc",
            Toast::CardReadOnly => "Read-only: SELECT+START",
            Toast::CardRepaired => "Card repaired",
            Toast::CardRepairFailed => "Repair failed",
            Toast::CardRepairLowBattery => "Charge to 20% first",
            Toast::PickAtMost => "5 games at most",
            Toast::PickAtLeast => "Pick 1 game at least",
            Toast::NoGuide => "No guide found",
            Toast::Palette(colors, _) => colors.label(),
            Toast::Disc(n) => DISC_IN
                .get(usize::from(n).wrapping_sub(1))
                .copied()
                .unwrap_or("Disc inserted"),
        }
    }

    /// The toast as the kit draws it, in `pal`.
    pub fn face(self, pal: &Palette) -> UndoFace {
        toast_face(self.label(), self.text(), pal)
    }
}
