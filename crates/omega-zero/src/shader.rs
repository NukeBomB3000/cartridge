//! The screen look each system's games are drawn through, kept per system: on a handheld, with
//! its colour correction on or off.

use std::path::Path;

use omega_gfx::Look;
use omega_store::{ini, Platform};

pub const SHADER_FILE: &str = "Config/shader.ini";

/// How a look with the colour correction on is kept, after the look's own name.
const CC: &str = "_cc";

/// The looks SELECT+X goes round for `platform`, none first: a handheld's LCD stripes, pixel
/// grid and round dots, or a television's scanlines and CRT.
pub fn looks_for(platform: Platform) -> &'static [Look] {
    match platform {
        Platform::Gba | Platform::Gb | Platform::Gbc => {
            &[Look::Off, Look::Lcd, Look::Grid, Look::Dot]
        }
        Platform::Snes | Platform::Ps1 | Platform::Pce | Platform::Nes | Platform::Md => {
            &[Look::Off, Look::Scanlines, Look::Crt]
        }
    }
}

/// The look and colour correction after `look` and `cc` for `platform`: on a handheld each look
/// comes without the correction and then with it, round to none without it after the last; a
/// television's console has no correction to go round.
pub fn next_look(platform: Platform, look: Look, cc: bool) -> (Look, bool) {
    if platform.colour_correction() && !cc {
        return (look, true);
    }
    let looks = looks_for(platform);
    let at = looks.iter().position(|l| *l == look).unwrap_or(0);
    (looks[(at + 1) % looks.len()], false)
}

/// The look every game of `platform` is drawn through: the one its system was last left in,
/// kept under the system's folder name, else none. A look the system does not offer is none.
pub fn shader_for(root: &Path, platform: Platform) -> Look {
    kept(root, platform)
        .and_then(|(name, _)| Look::parse(&name))
        .filter(|l| looks_for(platform).contains(l))
        .unwrap_or_default()
}

/// Whether the colour correction was on in the look a handheld was last left in, if one was
/// kept for it.
pub fn colour_for(root: &Path, platform: Platform) -> Option<bool> {
    platform
        .colour_correction()
        .then(|| kept(root, platform).map(|(_, cc)| cc))
        .flatten()
}

fn kept(root: &Path, platform: Platform) -> Option<(String, bool)> {
    let value = ini::value(root, SHADER_FILE, platform.dir_name())?;
    Some(match value.strip_suffix(CC) {
        Some(name) => (name.to_string(), true),
        None => (value, false),
    })
}

pub fn write_shader(root: &Path, platform: Platform, look: Look, cc: bool) -> std::io::Result<()> {
    let value = match cc && platform.colour_correction() {
        true => format!("{}{CC}", look.as_str()),
        false => look.as_str().to_string(),
    };
    ini::write(root, SHADER_FILE, platform.dir_name(), &value)
}
