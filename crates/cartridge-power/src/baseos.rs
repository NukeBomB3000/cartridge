use std::path::Path;

pub const BASEOS_RELEASE: &str = "/etc/baseos-release";

/// What BaseOS says about the handheld it was built for, from /etc/baseos-release. Each
/// BaseOS image is built for one model, so this names it exactly.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BaseosRelease {
    /// The build target, such as `rg35xxsp` or `rgsp`.
    pub target: Option<String>,
    /// The model's name, such as `RG35XX SP`.
    pub model: Option<String>,
    pub version: Option<String>,
    /// How far the picture is turned counter-clockwise to stand upright on the panel.
    pub rotation_ccw: u32,
    pub wifi: Option<bool>,
    pub bluetooth: Option<bool>,
}

pub fn parse_baseos_release(text: &str) -> BaseosRelease {
    let mut r = BaseosRelease::default();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"').trim();
        let text = (!value.is_empty()).then(|| value.to_string());
        let flag = || match value.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        };
        match key.trim() {
            "BASEOS_TARGET" => r.target = text,
            "BASEOS_MODEL" => r.model = text,
            "BASEOS_VERSION" => r.version = text,
            "BASEOS_PANEL_ROTATION_CCW" => r.rotation_ccw = value.parse().unwrap_or(0) % 360,
            "BASEOS_WIFI" => r.wifi = flag(),
            "BASEOS_BLUETOOTH" => r.bluetooth = flag(),
            _ => {}
        }
    }
    r
}

/// The running BaseOS's description of its handheld, if this is BaseOS.
pub fn baseos_release() -> Option<BaseosRelease> {
    read_baseos_release(Path::new(BASEOS_RELEASE))
}

pub fn read_baseos_release(path: &Path) -> Option<BaseosRelease> {
    std::fs::read_to_string(path)
        .ok()
        .map(|t| parse_baseos_release(&t))
}
