//! The music on the card: `Music/`, opened one folder at a time. A folder with folders in it is
//! a place to go into; a folder with only songs in it is a tape to play. Every folder is one
//! tape on the Music shelf, named as the folder is.

use std::path::{Path, PathBuf};

use crate::platform::Platform;
use crate::scan::{is_hidden, sort_key, Cart};

/// The folder the music is kept in, under the card's root.
pub const MUSIC_DIR: &str = "Music";

/// The songs the player decodes.
pub const EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "oga", "wav", "aac", "m4a"];

/// How deep a folder is searched for songs before it counts as empty: a tape of albums of
/// discs is as far as anyone keeps them.
const SEARCH_DEPTH: u32 = 6;

/// What a folder holds: the folders to go into and the songs in it, each in the order it is
/// listed in.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Contents {
    pub folders: Vec<(String, PathBuf)>,
    pub songs: Vec<PathBuf>,
}

pub fn is_song(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| EXTENSIONS.iter().any(|k| ext.eq_ignore_ascii_case(k)))
}

/// Whether `dir` has a song in it, or in a folder in it, as deep as [`SEARCH_DEPTH`].
fn has_songs(dir: &Path, depth: u32) -> bool {
    let Ok(read) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if is_hidden(&path) {
            continue;
        }
        if path.is_file() && is_song(&path) {
            return true;
        }
        if path.is_dir() && depth > 0 && has_songs(&path, depth - 1) {
            return true;
        }
    }
    false
}

/// What `dir` holds: its folders with songs somewhere in them, by name, and the songs directly
/// in it, by name.
pub fn contents(dir: &Path) -> Contents {
    let mut out = Contents::default();
    let Ok(read) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if is_hidden(&path) {
            continue;
        }
        if path.is_dir() {
            if has_songs(&path, SEARCH_DEPTH) {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    out.folders.push((name.to_string(), path));
                }
            }
        } else if path.is_file() && is_song(&path) {
            out.songs.push(path);
        }
    }
    out.folders.sort_by_key(|(name, _)| sort_key(name));
    out.songs
        .sort_by_key(|p| sort_key(p.file_stem().and_then(|s| s.to_str()).unwrap_or_default()));
    out
}

/// The folder `path` names, under the card's `Music/`: each name goes one folder deeper.
pub fn folder(root: &Path, path: &[String]) -> PathBuf {
    path.iter()
        .fold(root.join(MUSIC_DIR), |dir, name| dir.join(name))
}

/// The tapes of the folder `path` names: one for each folder in it that has songs, or, where
/// it has no folders but songs, one for the folder itself, so a card with its songs loose in
/// `Music/` still has a tape.
pub fn tapes(root: &Path, path: &[String]) -> Vec<Cart> {
    let dir = folder(root, path);
    let held = contents(&dir);
    // a tape is coloured as a game is, by Config/cart_shell.ini, under its folder's name
    let read = |file: &str| std::fs::read_to_string(root.join(file)).unwrap_or_default();
    let shells = crate::cart_shell::layered(
        &read(crate::CART_SHELL_FILE),
        &read(crate::LABELS_SHELL_FILE),
    );
    let tape = |stem: String, rom: PathBuf| Cart {
        platform: Platform::Music,
        shell: shells.get(&crate::cart_shell::key(&stem)).copied(),
        stem,
        rom,
        label: None,
        title: String::new(),
        code: String::new(),
    };
    if held.folders.is_empty() && !held.songs.is_empty() {
        let stem = path
            .last()
            .cloned()
            .unwrap_or_else(|| MUSIC_DIR.to_string());
        return vec![tape(stem, dir)];
    }
    held.folders
        .into_iter()
        .map(|(name, rom)| tape(name, rom))
        .collect()
}

/// Whether a tape's folder is played, not opened: it has songs and no folder to go into. The
/// songs of a folder that also has folders in it are not part of its tapes.
pub fn plays(tape: &Path) -> bool {
    let held = contents(tape);
    held.folders.is_empty() && !held.songs.is_empty()
}

/// The songs a tape plays, in the order they are listed in: those of its folder.
pub fn songs(tape: &Path) -> Vec<PathBuf> {
    contents(tape).songs
}
