//! The screen look each system's games are drawn through, kept per system.

use std::path::Path;

use cartridge_gfx::Look;
use cartridge_store::{ini, Platform};

pub const SHADER_FILE: &str = "Config/shader.ini";

/// The looks SELECT+X goes round for `platform`, none first: a handheld's LCD stripes, pixel
/// grid, round dots, paper and see-through pixels, and on the Game Boy its greens; or a
/// television's scanlines and CRT.
pub fn looks_for(platform: Platform) -> &'static [Look] {
    const HANDHELD: [Look; 6] = [
        Look::Off,
        Look::Lcd,
        Look::Grid,
        Look::Dot,
        Look::Simpletex,
        Look::Transparent,
    ];
    match platform {
        Platform::Gb => &[
            Look::Off,
            Look::Lcd,
            Look::Grid,
            Look::Dot,
            Look::Simpletex,
            Look::Transparent,
            Look::Dmg,
        ],
        Platform::Gba | Platform::Gbc | Platform::Gg => &HANDHELD,
        Platform::Snes
        | Platform::Ps1
        | Platform::Pce
        | Platform::Nes
        | Platform::Md
        | Platform::Sms => &[Look::Off, Look::Scanlines, Look::Crt, Look::Curved],
        // two screens scaled by no whole number, which a pixel look would cut unevenly
        Platform::Nds => &[Look::Off],
        Platform::Music | Platform::Video => &[Look::Off],
    }
}

/// The look after `look` for `platform`, round to none after the last.
pub fn next_look(platform: Platform, look: Look) -> Look {
    let looks = looks_for(platform);
    let at = looks.iter().position(|l| *l == look).unwrap_or(0);
    looks[(at + 1) % looks.len()]
}

/// The look every game of `platform` is drawn through: the one its system was last left in,
/// kept under the system's folder name, else none. A look the system does not offer is none.
pub fn shader_for(root: &Path, platform: Platform) -> Look {
    ini::value(root, SHADER_FILE, platform.dir_name())
        .as_deref()
        .and_then(Look::parse)
        .filter(|l| looks_for(platform).contains(l))
        .unwrap_or_default()
}

pub fn write_shader(root: &Path, platform: Platform, look: Look) -> std::io::Result<()> {
    ini::write(root, SHADER_FILE, platform.dir_name(), look.as_str())
}
