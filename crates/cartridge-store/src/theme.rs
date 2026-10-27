use std::path::Path;

pub const THEME_FILE: &str = "theme.txt";

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Theme {
    pub housing: [u8; 3],
    pub edge: [u8; 3],
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            housing: [0x24, 0x24, 0x29],
            edge: [0x4d, 0x4d, 0x57],
        }
    }
}

impl Theme {
    pub fn read(root: &Path) -> Self {
        Self::read_override(root).unwrap_or_default()
    }

    /// The card's Config/theme.txt, when it has one: the shelf's chrome is then drawn in it
    /// rather than in the colours the player picked.
    pub fn read_override(root: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(root.join("Config").join(THEME_FILE)).ok()?;
        Some(Self::parse(&text))
    }

    pub fn parse(text: &str) -> Self {
        let mut theme = Theme::default();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let (Some(name), Some(value)) = (parts.next(), parts.next()) else {
                continue;
            };
            if parts.next().is_some() {
                continue;
            }
            let Some(rgb) = hex(value) else {
                continue;
            };
            match name.to_ascii_lowercase().as_str() {
                "housing" => theme.housing = rgb,
                "edge" => theme.edge = rgb,
                _ => {}
            }
        }
        theme
    }
}

fn hex(value: &str) -> Option<[u8; 3]> {
    let digits = value.strip_prefix('#').unwrap_or(value);
    if digits.len() != 6 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}
