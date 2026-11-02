//! Handing the screen to a program of its own. PPSSPP runs the PSP's games and wants the
//! framebuffer and the sound for itself, so the frontend writes down which game and ends;
//! `launch_frontend.sh` finds the request, runs `System/psp.sh` on the game and starts the
//! frontend again when it ends, which comes back to the shelf at the cart it left.

use std::io;
use std::path::{Path, PathBuf};

/// Where the request is kept: a file in the run folder, gone with the next boot.
pub fn request_file() -> PathBuf {
    let run = std::env::var_os("AGS_RUN").map_or_else(|| PathBuf::from("/run"), PathBuf::from);
    run.join("cartridge-handoff")
}

/// Notes that the program `program` is to run `game` next, through a file of its own so the
/// script never reads half of it.
pub fn request(file: &Path, program: &str, game: &Path) -> io::Result<()> {
    let tmp = file.with_extension("tmp");
    std::fs::write(&tmp, format!("{program}\n{}\n", game.display()))?;
    std::fs::rename(&tmp, file)
}

/// Where `psp.sh` leaves a note that the handheld is to power off once cartridge is back: a
/// game left through the power button or an empty battery, saved, with its cart still in the
/// slot, so that the next start goes into it.
pub fn poweroff_file() -> PathBuf {
    request_file().with_file_name("cartridge-poweroff")
}
