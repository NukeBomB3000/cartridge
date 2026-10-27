use std::io;
use std::path::Path;

pub const CONFIG_DIR: &str = "Config";

const MOVED: [&str; 5] = [
    "cartridge.state",
    "selected_core.ini",
    "video_mode.ini",
    "cart_shell.ini",
    "theme.txt",
];

/// The state file's name while the frontend was omega-zero.
const OLD_STATE: &str = "omega-zero.state";

pub fn move_config(root: &Path) -> io::Result<()> {
    let config = root.join(CONFIG_DIR);
    std::fs::create_dir_all(&config)?;
    for name in MOVED {
        let old = root.join("System").join(name);
        let new = config.join(name);
        if old.is_file() && !new.exists() {
            std::fs::rename(&old, &new)?;
        }
    }
    // the state file from before the rename to Cartridge, in either place it was kept
    let new = config.join("cartridge.state");
    for old in [config.join(OLD_STATE), root.join("System").join(OLD_STATE)] {
        if old.is_file() && !new.exists() {
            std::fs::rename(&old, &new)?;
        }
    }
    Ok(())
}
