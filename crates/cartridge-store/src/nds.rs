//! A Nintendo DS ROM's header: its title, twelve capitals at the start, and the four-letter
//! game code after it, as GBATEK lays them out.

use std::io::Read;
use std::path::Path;

/// The title and the game code a DS ROM's header gives, each trimmed of padding.
pub fn header(rom: &Path) -> Option<(String, String)> {
    let mut head = [0u8; 16];
    std::fs::File::open(rom).ok()?.read_exact(&mut head).ok()?;
    let text = |bytes: &[u8]| {
        bytes
            .iter()
            .take_while(|&&b| b != 0)
            .filter(|b| b.is_ascii_graphic() || **b == b' ')
            .map(|&b| b as char)
            .collect::<String>()
            .trim()
            .to_string()
    };
    Some((text(&head[..12]), text(&head[12..16])))
}
