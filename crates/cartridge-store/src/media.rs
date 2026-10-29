//! The media on the card, `Music/` and `Videos/`, opened one folder at a time. A folder with
//! folders in it is a place to go into; a folder with only files in it is a tape to play. Every
//! folder is one tape on its shelf, named as the folder is: a cassette for the music, a VHS
//! cassette for the videos.

use std::path::{Path, PathBuf};

use crate::platform::Platform;
use crate::scan::{is_hidden, sort_key, Cart};

/// What is kept: music or videos.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Music,
    Video,
    /// Pictures: a folder of them is a Polaroid, which the viewer opens.
    Pictures,
    /// Books: each EPUB is a book, which the reader opens.
    Books,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Music, Kind::Video, Kind::Pictures, Kind::Books];

    /// The folder it is kept in, under the card's root.
    pub fn dir(self) -> &'static str {
        match self {
            Kind::Music => "Music",
            Kind::Video => "Videos",
            Kind::Pictures => "Pictures",
            Kind::Books => "Books",
        }
    }

    /// The files the player decodes.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Kind::Music => &["mp3", "flac", "ogg", "oga", "wav", "aac", "m4a"],
            Kind::Video => &[
                "mp4", "m4v", "mkv", "avi", "mov", "webm", "ts", "mpg", "mpeg", "wmv", "flv",
            ],
            Kind::Pictures => &["jpg", "jpeg", "png"],
            Kind::Books => &["epub", "txt"],
        }
    }

    /// The shelf its tapes stand on.
    pub fn platform(self) -> Platform {
        match self {
            Kind::Music => Platform::Music,
            Kind::Video => Platform::Video,
            Kind::Pictures => Platform::Pictures,
            Kind::Books => Platform::Books,
        }
    }

    /// The kind whose shelf is `platform`'s, if it is one.
    pub fn of(platform: Platform) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.platform() == platform)
    }

    pub fn is_file(self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                self.extensions()
                    .iter()
                    .any(|k| ext.eq_ignore_ascii_case(k))
            })
    }
}

/// How deep a folder is searched for files before it counts as empty: a tape of albums of
/// discs is as far as anyone keeps them.
const SEARCH_DEPTH: u32 = 6;

/// What a folder holds: the folders to go into and the files in it, each in the order it is
/// listed in.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Contents {
    pub folders: Vec<(String, PathBuf)>,
    pub files: Vec<PathBuf>,
}

/// Whether `dir` has a file of `kind` in it, or in a folder in it, as deep as [`SEARCH_DEPTH`].
fn has_files(kind: Kind, dir: &Path, depth: u32) -> bool {
    let Ok(read) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if is_hidden(&path) {
            continue;
        }
        if path.is_file() && kind.is_file(&path) {
            return true;
        }
        if path.is_dir() && depth > 0 && has_files(kind, &path, depth - 1) {
            return true;
        }
    }
    false
}

/// What `dir` holds: its folders with files of `kind` somewhere in them, by name, and the files
/// directly in it, by name.
pub fn contents(kind: Kind, dir: &Path) -> Contents {
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
            if has_files(kind, &path, SEARCH_DEPTH) {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    out.folders.push((name.to_string(), path));
                }
            }
        } else if path.is_file() && kind.is_file(&path) {
            out.files.push(path);
        }
    }
    out.folders.sort_by_key(|(name, _)| sort_key(name));
    out.files
        .sort_by_key(|p| sort_key(p.file_stem().and_then(|s| s.to_str()).unwrap_or_default()));
    out
}

/// The folder `path` names, under the card's folder for `kind`: each name goes one folder deeper.
pub fn folder(kind: Kind, root: &Path, path: &[String]) -> PathBuf {
    path.iter()
        .fold(root.join(kind.dir()), |dir, name| dir.join(name))
}

/// The picture a tape wears on its face: a folder of pictures wears its first one, a book its
/// cover (cut from the EPUB, or the card's own `Labels/Books/<name>.png`).
fn cover(kind: Kind, root: &Path, stem: &str, rom: &Path) -> Option<PathBuf> {
    match kind {
        Kind::Pictures => first_file(kind, rom, 4),
        Kind::Books => {
            let own = root
                .join("Labels")
                .join(kind.dir())
                .join(format!("{stem}.png"));
            if own.is_file() {
                return Some(own);
            }
            crate::epub::cached_cover(root, rom)
        }
        _ => None,
    }
}

/// The first file of `kind` in `dir`, or in the first folder in it that has one, as deep as
/// `depth`.
pub fn first_file(kind: Kind, dir: &Path, depth: u32) -> Option<PathBuf> {
    let held = contents(kind, dir);
    if let Some(file) = held.files.into_iter().next() {
        return Some(file);
    }
    if depth == 0 {
        return None;
    }
    held.folders
        .into_iter()
        .find_map(|(_, folder)| first_file(kind, &folder, depth - 1))
}

/// The tapes of the folder `path` names. Music: one for each folder in it that has songs, or,
/// where it has no folders but songs, one for the folder itself, which plays its songs as an
/// album, so a card with its songs loose in `Music/` still has a tape. Videos: one for each
/// folder in it that has videos, and one for each video in it, a film to a tape.
pub fn tapes(kind: Kind, root: &Path, path: &[String]) -> Vec<Cart> {
    let dir = folder(kind, root, path);
    let held = contents(kind, &dir);
    // a tape is coloured as a game is, by Config/cart_shell.ini, under its name
    let read = |file: &str| std::fs::read_to_string(root.join(file)).unwrap_or_default();
    let shells = crate::cart_shell::layered(
        &read(crate::CART_SHELL_FILE),
        &read(crate::LABELS_SHELL_FILE),
    );
    let tape = |stem: String, rom: PathBuf| Cart {
        platform: kind.platform(),
        shell: shells.get(&crate::cart_shell::key(&stem)).copied(),
        label: cover(kind, root, &stem, &rom),
        stem,
        rom,
        title: String::new(),
        code: String::new(),
    };
    if matches!(kind, Kind::Music | Kind::Pictures)
        && held.folders.is_empty()
        && !held.files.is_empty()
    {
        let stem = path
            .last()
            .cloned()
            .unwrap_or_else(|| kind.dir().to_string());
        return vec![tape(stem, dir)];
    }
    let mut out: Vec<Cart> = held
        .folders
        .into_iter()
        .map(|(name, rom)| tape(name, rom))
        .collect();
    if matches!(kind, Kind::Video | Kind::Books) {
        out.extend(held.files.into_iter().map(|file| {
            let stem = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            tape(stem, file)
        }));
    }
    out
}

/// Whether a tape is played, not opened. A video's file is, and so is the folder of just one
/// video; a folder of songs with no folder to go into is, as an album. The files of a folder that
/// also has folders in it are not part of its tapes.
pub fn plays(kind: Kind, tape: &Path) -> bool {
    if tape.is_file() {
        return true;
    }
    let held = contents(kind, tape);
    match kind {
        Kind::Music | Kind::Pictures => held.folders.is_empty() && !held.files.is_empty(),
        Kind::Video | Kind::Books => held.folders.is_empty() && held.files.len() == 1,
    }
}

/// The files a tape plays, in the order they are listed in: a file's own, or those of its folder.
pub fn files(kind: Kind, tape: &Path) -> Vec<PathBuf> {
    if tape.is_file() {
        return vec![tape.to_path_buf()];
    }
    contents(kind, tape).files
}
