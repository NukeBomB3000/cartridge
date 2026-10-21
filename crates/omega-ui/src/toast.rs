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
    /// SELECT+Y on a television's console: the picture at its actual size, a whole scale.
    PictureActual,
    /// SELECT+Y on a television's console: the picture a television's 4:3.
    PictureAspect,
    /// SELECT+Y on a television's console: the picture stretched over the screen.
    PictureStretch,
    /// SELECT+L2 on a handheld: the picture as it was always drawn.
    PictureOriginal,
    /// SELECT+L2 on a handheld: the picture its own shape, as large as the screen holds.
    PictureFit,
    /// SELECT+X in a game: the screen look now chosen for its system.
    Shader(omega_gfx::Look, bool),
    /// SELECT + MENU on a PlayStation game on one disc: there is no other to swap in.
    OneDisc,
    /// The DISC screen swapped disc `n`, counting from 1, into the drive.
    Disc(u8),
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
    pub const ALL: [Toast; 17] = [
        Toast::StateSaved,
        Toast::StateLoaded,
        Toast::NeedsGpsp,
        Toast::NoLink,
        Toast::LinkEnded,
        Toast::PeerEnded,
        Toast::BiosMismatch,
        Toast::ColourOn,
        Toast::ColourOff,
        Toast::PictureActual,
        Toast::PictureAspect,
        Toast::PictureStretch,
        Toast::PictureOriginal,
        Toast::PictureFit,
        Toast::Shader(omega_gfx::Look::Off, false),
        Toast::OneDisc,
        Toast::Disc(1),
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
            Toast::PictureActual => 9,
            Toast::PictureAspect => 10,
            Toast::PictureStretch => 11,
            Toast::PictureOriginal => 12,
            Toast::PictureFit => 13,
            Toast::Shader(..) => 14,
            Toast::OneDisc => 15,
            Toast::Disc(_) => 16,
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
            Toast::PictureActual
            | Toast::PictureAspect
            | Toast::PictureStretch
            | Toast::PictureOriginal
            | Toast::PictureFit => "PICTURE",
            Toast::Shader(..) => "SHADER",
            Toast::OneDisc | Toast::Disc(_) => "DISC",
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
            Toast::PictureActual => "Integer scale",
            Toast::PictureAspect => "Keep aspect 4:3",
            Toast::PictureStretch => "Stretch",
            Toast::PictureOriginal => "Original",
            Toast::PictureFit => "Keep aspect",
            Toast::Shader(look, cc) => match (look, cc) {
                (omega_gfx::Look::Off, false) => "Off",
                (omega_gfx::Look::Off, true) => "Off CC",
                (omega_gfx::Look::Lcd, false) => "LCD",
                (omega_gfx::Look::Lcd, true) => "LCD CC",
                (omega_gfx::Look::Grid, false) => "Grid",
                (omega_gfx::Look::Grid, true) => "Grid CC",
                (omega_gfx::Look::Dot, false) => "Dot",
                (omega_gfx::Look::Dot, true) => "Dot CC",
                (omega_gfx::Look::Simpletex, false) => "Simpletex",
                (omega_gfx::Look::Simpletex, true) => "Simpletex CC",
                (omega_gfx::Look::Transparent, false) => "Transparent",
                (omega_gfx::Look::Transparent, true) => "Transparent CC",
                (omega_gfx::Look::Dmg, _) => "DMG",
                (omega_gfx::Look::Scanlines, _) => "Scanlines",
                (omega_gfx::Look::Crt, _) => "CRT",
            },
            Toast::OneDisc => "Only one disc",
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
