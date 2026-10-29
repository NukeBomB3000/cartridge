//! The music on the card: `Music/`, its folders the tapes of the Music shelf (see [`crate::media`]).

use std::path::{Path, PathBuf};

use crate::media::{self, Kind};
use crate::scan::Cart;

/// The folder the music is kept in, under the card's root.
pub const MUSIC_DIR: &str = "Music";

/// The songs the player decodes.
pub const EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "oga", "wav", "aac", "m4a"];

pub use crate::media::Contents;

pub fn is_song(path: &Path) -> bool {
    Kind::Music.is_file(path)
}

pub fn contents(dir: &Path) -> Contents {
    media::contents(Kind::Music, dir)
}

pub fn folder(root: &Path, path: &[String]) -> PathBuf {
    media::folder(Kind::Music, root, path)
}

pub fn tapes(root: &Path, path: &[String]) -> Vec<Cart> {
    media::tapes(Kind::Music, root, path)
}

pub fn plays(tape: &Path) -> bool {
    media::plays(Kind::Music, tape)
}

pub fn songs(tape: &Path) -> Vec<PathBuf> {
    media::files(Kind::Music, tape)
}
