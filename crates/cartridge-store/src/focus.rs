//! Five Games: the handheld kept to up to five games picked from the shelves, the one shelf it
//! shows until Five Games is switched off. The games are kept one to a line in
//! `Config/five_games.txt` as `<system>/<file name without its extension>`, in the order they
//! stand; the file is there only while Five Games is on.

use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::platform::Platform;

/// The most games Five Games keeps.
pub const FOCUS_MAX: usize = 5;

pub const FOCUS_FILE: &str = "Config/five_games.txt";

fn focus_path(root: &Path) -> PathBuf {
    root.join(FOCUS_FILE)
}

/// The games Five Games keeps the handheld to, in the order they stand, when it is on.
pub fn read_focus(root: &Path) -> Option<Vec<(Platform, String)>> {
    let text = std::fs::read_to_string(focus_path(root)).ok()?;
    let games: Vec<(Platform, String)> = text
        .lines()
        .filter_map(|line| {
            let (dir, stem) = line.trim().split_once('/')?;
            let platform = Platform::ALL.into_iter().find(|p| p.dir_name() == dir)?;
            (!stem.is_empty()).then(|| (platform, stem.to_string()))
        })
        .take(FOCUS_MAX)
        .collect();
    Some(games)
}

/// Switches Five Games on with `games`, or off with none.
pub fn write_focus(root: &Path, games: Option<&[(Platform, String)]>) -> std::io::Result<()> {
    match games {
        Some(games) => {
            let text: String = games
                .iter()
                .take(FOCUS_MAX)
                .map(|(p, stem)| format!("{}/{stem}\n", p.dir_name()))
                .collect();
            let path = focus_path(root);
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            atomic_write(&path, text.as_bytes())
        }
        None => match std::fs::remove_file(focus_path(root)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_games_are_kept_in_their_order_and_switched_off_by_going() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(read_focus(d.path()), None, "on before it was switched on");
        let games = vec![
            (Platform::Gba, "Advance Wars".to_string()),
            (Platform::Snes, "Super Mario World (USA)".to_string()),
        ];
        write_focus(d.path(), Some(&games)).unwrap();
        assert_eq!(read_focus(d.path()), Some(games));
        write_focus(d.path(), None).unwrap();
        assert_eq!(read_focus(d.path()), None);
        write_focus(d.path(), None).unwrap();
    }

    #[test]
    fn no_more_than_five_are_kept() {
        let d = tempfile::tempdir().unwrap();
        let games: Vec<_> = (0..7)
            .map(|i| (Platform::Nes, format!("Game {i}")))
            .collect();
        write_focus(d.path(), Some(&games)).unwrap();
        assert_eq!(read_focus(d.path()).unwrap().len(), FOCUS_MAX);
    }
}
