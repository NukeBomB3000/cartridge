//! How the analog sticks' RGB lights glow, as Settings > Theme > Stick Lights picks. Only the
//! RG40XX and the CubeXX have them.

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum StickLights {
    /// Dark, as BaseOS leaves them.
    #[default]
    Off,
    /// Steady, in the theme's colour.
    Colour,
    /// Slowly fading in and out, in the theme's colour.
    Breathe,
    /// Cycling through every colour.
    Rainbow,
}

impl StickLights {
    pub const ALL: [StickLights; 4] = [
        StickLights::Off,
        StickLights::Colour,
        StickLights::Breathe,
        StickLights::Rainbow,
    ];

    pub fn key(self) -> &'static str {
        match self {
            StickLights::Off => "off",
            StickLights::Colour => "colour",
            StickLights::Breathe => "breathe",
            StickLights::Rainbow => "rainbow",
        }
    }

    pub fn from_key(key: &str) -> Option<StickLights> {
        Self::ALL.into_iter().find(|l| l.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            StickLights::Off => "Off",
            StickLights::Colour => "Colour",
            StickLights::Breathe => "Breathe",
            StickLights::Rainbow => "Rainbow",
        }
    }
}
