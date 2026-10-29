//! The videos on the card: `Videos/`, its folders the tapes of the Video shelf (see
//! [`crate::media`]).

use std::path::{Path, PathBuf};

use crate::media::{self, Kind};
use crate::scan::Cart;

/// The folder the videos are kept in, under the card's root.
pub const VIDEO_DIR: &str = "Videos";

pub fn is_video(path: &Path) -> bool {
    Kind::Video.is_file(path)
}

pub fn tapes(root: &Path, path: &[String]) -> Vec<Cart> {
    media::tapes(Kind::Video, root, path)
}

pub fn plays(tape: &Path) -> bool {
    media::plays(Kind::Video, tape)
}

pub fn videos(tape: &Path) -> Vec<PathBuf> {
    media::files(Kind::Video, tape)
}
