//! Where a song or a film was left: kept in `Config/media_resume.ini`, a line for each tape of
//! music (its folder, the song it was at and how far into it) and for each video (its file, how
//! far into it). A short A on a tape goes on from there, a long A starts again.

use std::path::Path;

use cartridge_store::ini;

pub const MEDIA_RESUME_FILE: &str = "Config/media_resume.ini";

/// A video that stands this close to its end counts as watched: it starts again.
const WATCHED_BEFORE_END_MS: u64 = 30_000;
/// A place this near the start is not worth going back to.
const WORTH_RESUMING_MS: u64 = 5_000;

/// The line a tape or a file is kept under: its path from the card's root.
pub fn key(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// The song (from 0) and the millisecond kept for `key`.
pub fn read(root: &Path, key: &str) -> Option<(usize, u64)> {
    let value = ini::value(root, MEDIA_RESUME_FILE, key)?;
    let (index, ms) = value.split_once(':')?;
    Some((index.trim().parse().ok()?, ms.trim().parse().ok()?))
}

pub fn write(root: &Path, key: &str, index: usize, ms: u64) {
    if let Err(e) = ini::write(root, MEDIA_RESUME_FILE, key, &format!("{index}:{ms}")) {
        eprintln!("cartridge: {MEDIA_RESUME_FILE}: {e}");
    }
}

pub fn clear(root: &Path, key: &str) {
    let _ = ini::remove(root, MEDIA_RESUME_FILE, key);
}

/// Where in a video of `duration_ms` to start from `kept`: the place kept, unless it is too near
/// either end to be one worth going back to (the length is not known when it is 0).
pub fn video_start(kept: Option<u64>, duration_ms: u64) -> u64 {
    match kept {
        Some(ms)
            if ms >= WORTH_RESUMING_MS
                && (duration_ms == 0 || ms + WATCHED_BEFORE_END_MS < duration_ms) =>
        {
            ms
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_place_is_kept_and_read_back_under_the_path_from_the_root() {
        let d = tempfile::tempdir().unwrap();
        let file = d.path().join("Videos/Series/e01.mkv");
        let k = key(d.path(), &file);
        assert_eq!(k, "Videos/Series/e01.mkv");
        assert_eq!(read(d.path(), &k), None);
        write(d.path(), &k, 0, 754_000);
        assert_eq!(read(d.path(), &k), Some((0, 754_000)));
        write(d.path(), "Music/Band/Album", 4, 61_000);
        assert_eq!(read(d.path(), "Music/Band/Album"), Some((4, 61_000)));
        assert_eq!(read(d.path(), &k), Some((0, 754_000)));
        clear(d.path(), &k);
        assert_eq!(read(d.path(), &k), None);
    }

    #[test]
    fn a_film_goes_on_unless_it_is_hardly_begun_or_all_but_over() {
        assert_eq!(video_start(Some(600_000), 5_400_000), 600_000);
        assert_eq!(video_start(Some(2_000), 5_400_000), 0, "hardly begun");
        assert_eq!(video_start(Some(5_390_000), 5_400_000), 0, "all but over");
        assert_eq!(video_start(None, 5_400_000), 0);
        assert_eq!(video_start(Some(600_000), 0), 600_000, "length unknown");
    }
}
