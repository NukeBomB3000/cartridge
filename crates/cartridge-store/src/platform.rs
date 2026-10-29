use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Platform {
    #[default]
    Gba,
    Gb,
    Gbc,
    Snes,
    Ps1,
    Pce,
    Nes,
    Md,
    Gg,
    Sms,
    /// The Nintendo DS, on DSperate: both its screens laid out in one picture by the core.
    Nds,
    /// Music, on tapes: not games, so no core runs them; the folders of `Music/` stand on this
    /// shelf and its player plays what is in them.
    Music,
    /// Videos, on VHS tapes: the folders of `Videos/` stand on this shelf and its player plays
    /// what is in them.
    Video,
}

impl Platform {
    pub const ALL: [Platform; 13] = [
        Platform::Gba,
        Platform::Gb,
        Platform::Gbc,
        Platform::Snes,
        Platform::Ps1,
        Platform::Pce,
        Platform::Nes,
        Platform::Md,
        Platform::Gg,
        Platform::Sms,
        Platform::Nds,
        Platform::Music,
        Platform::Video,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Platform::Gba => "Game Boy Advance",
            Platform::Gb => "Game Boy",
            Platform::Gbc => "Game Boy Color",
            Platform::Snes => "Super Nintendo",
            Platform::Ps1 => "PlayStation",
            Platform::Pce => "PC Engine",
            Platform::Nes => "Nintendo Entertainment System",
            Platform::Md => "Mega Drive",
            Platform::Gg => "Game Gear",
            Platform::Sms => "Master System",
            Platform::Nds => "Nintendo DS",
            Platform::Music => "Music",
            Platform::Video => "Video",
        }
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Platform::Gba => "GBA",
            Platform::Gb => "GB",
            Platform::Gbc => "GBC",
            Platform::Snes => "SNES",
            Platform::Ps1 => "PS1",
            Platform::Pce => "PCE",
            Platform::Nes => "NES",
            Platform::Md => "MD",
            Platform::Gg => "GG",
            Platform::Sms => "SMS",
            Platform::Nds => "NDS",
            Platform::Music => "Music",
            Platform::Video => "Videos",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gba => &["gba"],
            Platform::Gb | Platform::Gbc => &["gb", "gbc"],
            Platform::Snes => &["sfc", "smc"],
            Platform::Ps1 => &["cue", "chd", "pbp", "m3u"],
            Platform::Pce => &["pce"],
            Platform::Nes => &["nes"],
            Platform::Md => &["md", "gen", "bin"],
            Platform::Gg => &["gg"],
            Platform::Sms => &["sms"],
            Platform::Nds => &["nds"],
            Platform::Music => crate::music::EXTENSIONS,
            Platform::Video => crate::media::Kind::Video.extensions(),
        }
    }

    pub fn picture(self) -> (u32, u32) {
        match self {
            Platform::Gba => (240, 160),
            Platform::Gb | Platform::Gbc => (160, 144),
            Platform::Snes => (256, 224),
            Platform::Ps1 => (320, 240),
            Platform::Pce => (256, 240),
            Platform::Nes => (256, 224),
            Platform::Md => (320, 224),
            Platform::Gg => (160, 144),
            Platform::Sms => (256, 192),
            // the top screen half again as big over the bottom one, as DSperate lays them out
            Platform::Nds => (384, 480),
            // no picture: the player draws its own screen
            Platform::Music => (320, 240),
            Platform::Video => (320, 240),
        }
    }

    /// Whether held X/Y are turbo A/B: on the handhelds, the NES and the Master System, whose
    /// games have no X or Y.
    pub fn turbo_xy(self) -> bool {
        matches!(
            self,
            Platform::Gba
                | Platform::Gb
                | Platform::Gbc
                | Platform::Nes
                | Platform::Gg
                | Platform::Sms
        )
    }

    /// Whether the core's colour correction for a handheld's screen applies.
    pub fn colour_correction(self) -> bool {
        matches!(self, Platform::Gba | Platform::Gb | Platform::Gbc)
    }

    /// Whether the handheld LCD's ghosting applies: the cores that run the handhelds' games,
    /// mGBA and gpSP, can blend each frame with the last, as Genesis Plus GX can a Game Gear's.
    pub fn lcd_ghosting(self) -> bool {
        matches!(
            self,
            Platform::Gba | Platform::Gb | Platform::Gbc | Platform::Gg
        )
    }

    /// The CPU clock a game of this system is held to, in kHz, with room to spare in its
    /// busiest scenes, so the handheld runs cooler and longer on a charge; None for the
    /// systems that need the whole of it. Measured on the H700: a Game Boy's core needs under
    /// half a core at 480 MHz, a GBA's or a SNES's all of one.
    pub fn cpu_khz(self) -> Option<u32> {
        match self {
            Platform::Gb
            | Platform::Gbc
            | Platform::Gg
            | Platform::Sms
            | Platform::Nes
            | Platform::Pce => Some(720_000),
            Platform::Md => Some(1_008_000),
            Platform::Gba => Some(1_200_000),
            // DSperate barely keeps up at the top clock: 54 to 60 fps in NSMB and Mario Kart
            Platform::Snes | Platform::Ps1 | Platform::Nds => None,
            // decoding a song takes a fraction of a core
            Platform::Music => Some(720_000),
            // video decoding takes every core it can have
            Platform::Video => None,
        }
    }

    /// Whether held L2 rewinds: not on the PlayStation, whose games use L2 and whose states
    /// are too big for the rewind buffer, nor on the DS, whose are bigger still: its L2 swaps
    /// the screens instead.
    pub fn rewinds(self) -> bool {
        !matches!(self, Platform::Ps1 | Platform::Nds)
    }

    /// Whether L2/R2 belong to the game rather than to rewind and fast-forward: only the
    /// PlayStation's.
    pub fn game_owns_l2_r2(self) -> bool {
        self == Platform::Ps1
    }

    pub fn accepts(self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                self.extensions()
                    .iter()
                    .any(|k| ext.eq_ignore_ascii_case(k))
            })
    }
}
