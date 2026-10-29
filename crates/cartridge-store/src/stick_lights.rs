//! How the analog sticks' RGB lights glow, as Settings > Theme picks. Only the RG40XX and the
//! CubeXX have them.

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum StickLights {
    /// Dark, as BaseOS leaves them.
    #[default]
    Off,
    /// In the theme's colour, as bright as the display is.
    Theme,
    /// In a colour of their own, as bright as they are set.
    Colour,
}

impl StickLights {
    pub const ALL: [StickLights; 3] = [StickLights::Off, StickLights::Theme, StickLights::Colour];

    pub fn key(self) -> &'static str {
        match self {
            StickLights::Off => "off",
            StickLights::Theme => "theme",
            StickLights::Colour => "colour",
        }
    }

    pub fn from_key(key: &str) -> Option<StickLights> {
        Self::ALL.into_iter().find(|l| l.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            StickLights::Off => "Off",
            StickLights::Theme => "Theme",
            StickLights::Colour => "Colour",
        }
    }
}

/// The colours the lights can be set to: the name cartridge.state keeps, the name the row
/// prints, and the colour.
pub const LIGHT_COLOURS: [(&str, &str, [u8; 3]); 9] = [
    ("white", "White", [255, 255, 255]),
    ("red", "Red", [255, 0, 0]),
    ("orange", "Orange", [255, 90, 0]),
    ("yellow", "Yellow", [255, 200, 0]),
    ("green", "Green", [0, 255, 0]),
    ("cyan", "Cyan", [0, 220, 220]),
    ("blue", "Blue", [0, 80, 255]),
    ("purple", "Purple", [150, 0, 255]),
    ("pink", "Pink", [255, 40, 140]),
];

/// The lights' brightness steps, 1 to this, each a tenth.
pub const LIGHT_LEVEL_MAX: u8 = 10;

pub fn light_colour_index(key: &str) -> Option<u8> {
    LIGHT_COLOURS
        .iter()
        .position(|c| c.0 == key)
        .map(|i| i as u8)
}
