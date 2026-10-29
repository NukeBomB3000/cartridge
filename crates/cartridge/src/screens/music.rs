//! The Music shelf: the folders of `Music/` as tapes, one folder at a time. A on a tape whose
//! folder has folders in it goes into it, the shelf then showing those; a tape whose folder has
//! only songs in it plays them; B goes back out, to the tape that was open.

use cartridge_store::{music, Platform};
use cartridge_ui::Shelf;

use crate::app::{playful, App};

impl App {
    /// Whether the shelf in front is the music's.
    pub(crate) fn on_music_shelf(&self) -> bool {
        self.shelves
            .get(self.shelf_at)
            .is_some_and(|(p, _)| *p == Platform::Music)
    }

    /// Fills the Music shelf with the tapes of the folder `music_path` names, the cursor on
    /// the tape at `index`.
    pub(crate) fn fill_music(&mut self, index: usize) {
        let Some(root) = self.root.clone() else {
            return;
        };
        let Some(at) = self.shelves.iter().position(|(p, _)| *p == Platform::Music) else {
            return;
        };
        let carts = music::tapes(&root, &self.music_path);
        let mut shelf = playful(Shelf::new(carts));
        shelf.select(index);
        let old = std::mem::replace(&mut self.shelves[at].1, shelf);
        let mut old = old;
        self.dropped_faces.extend(old.take_faces());
        self.faces_stale = true;
        if at == self.shelf_at {
            self.shelf_platform = cartridge_ui::Printed::default();
            self.pill_letter = None;
            self.shelf_named = None;
            self.name_pending = true;
        }
    }

    /// A on a tape: its folder is opened, or its songs are played.
    pub(crate) fn open_tape(&mut self) {
        let Some(cart) = self.shelf().carts.get(self.shelf().index).cloned() else {
            return;
        };
        if music::plays(&cart.rom) {
            self.play_tape(&cart.stem, &cart.rom);
            return;
        }
        self.music_back.push(self.shelf().index);
        self.music_path.push(cart.stem);
        self.fill_music(0);
    }

    /// B on the Music shelf: out of the folder that is open, onto the tape it was.
    pub(crate) fn leave_tape(&mut self) {
        if self.music_path.pop().is_none() {
            return;
        }
        let at = self.music_back.pop().unwrap_or(0);
        self.fill_music(at);
    }

    /// The tape's songs, played in the order they are listed in.
    fn play_tape(&mut self, _name: &str, _folder: &std::path::Path) {
        // the player comes next: for now the tape is only opened
    }
}
