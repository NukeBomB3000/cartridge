//! The Music and Video shelves: the folders of `Music/` and `Videos/` as tapes, one folder at a
//! time. A on a tape whose folder has folders in it goes into it, the shelf then showing those; a
//! tape whose folder has only files in it plays them; B goes back out, to the tape that was open.

use cartridge_store::media::{self, Kind};
use cartridge_ui::Shelf;

use crate::app::{playful, App};

/// A kind's place in the arrays the shelves' folders are kept in.
pub(crate) fn slot(kind: Kind) -> usize {
    match kind {
        Kind::Music => 0,
        Kind::Video => 1,
    }
}

impl App {
    /// The kind of tapes on the shelf in front, if it is a media shelf.
    pub(crate) fn shelf_kind(&self) -> Option<Kind> {
        self.shelves
            .get(self.shelf_at)
            .and_then(|(p, _)| Kind::of(*p))
    }

    /// Whether the shelf in front is one of the tape shelves.
    pub(crate) fn on_media_shelf(&self) -> bool {
        self.shelf_kind().is_some()
    }

    /// Fills the shelf of `kind` with the tapes of the folder its path names, the cursor on the
    /// tape at `index`.
    pub(crate) fn fill_media(&mut self, kind: Kind, index: usize) {
        let Some(root) = self.root.clone() else {
            return;
        };
        let platform = kind.platform();
        let Some(at) = self.shelves.iter().position(|(p, _)| *p == platform) else {
            return;
        };
        let carts = media::tapes(kind, &root, &self.media_path[slot(kind)]);
        let mut shelf = playful(Shelf::new(carts));
        shelf.select(index);
        let mut old = std::mem::replace(&mut self.shelves[at].1, shelf);
        self.dropped_faces.extend(old.take_faces());
        self.media_stale.push(platform);
        if at == self.shelf_at {
            self.shelf_platform = cartridge_ui::Printed::default();
            self.pill_letter = None;
            self.shelf_named = None;
            self.name_pending = true;
        }
    }

    /// Goes back to the top of both shelves' folders, and fills them.
    pub(crate) fn fill_all_media(&mut self) {
        for kind in Kind::ALL {
            self.media_path[slot(kind)].clear();
            self.media_back[slot(kind)].clear();
            self.fill_media(kind, 0);
        }
    }

    /// The tapes whose faces have to be drawn, taken once.
    pub fn take_media_stale(&mut self) -> Vec<cartridge_store::Platform> {
        let mut stale = std::mem::take(&mut self.media_stale);
        stale.dedup();
        stale
    }

    /// The tapes of the shelf of `platform` now.
    pub fn media_carts(&self, platform: cartridge_store::Platform) -> Vec<cartridge_store::Cart> {
        self.shelves
            .iter()
            .find(|(p, _)| *p == platform)
            .map(|(_, shelf)| shelf.carts.clone())
            .unwrap_or_default()
    }

    /// The faces the tapes of the shelf of `platform` are drawn on, in their order.
    pub fn set_media_faces(
        &mut self,
        platform: cartridge_store::Platform,
        faces: Vec<cartridge_ui::TexId>,
    ) {
        if let Some((_, shelf)) = self.shelves.iter_mut().find(|(p, _)| *p == platform) {
            shelf.set_faces(faces);
        }
    }

    /// A on a tape: its folder is opened, or its files are played, from where they were left,
    /// or from the first with a long A (`clean`).
    pub(crate) fn open_tape(&mut self, clean: bool) {
        let Some(kind) = self.shelf_kind() else {
            return;
        };
        let Some(cart) = self.shelf().carts.get(self.shelf().index).cloned() else {
            return;
        };
        if media::plays(kind, &cart.rom) {
            match kind {
                Kind::Music => self.play_tape(&cart.stem, &cart.rom, clean),
                Kind::Video => self.play_video(&cart.stem, &cart.rom, clean),
            }
            return;
        }
        let n = slot(kind);
        self.media_back[n].push(self.shelf().index);
        self.media_path[n].push(cart.stem);
        self.fill_media(kind, 0);
    }

    /// B on a tape shelf: out of the folder that is open, onto the tape it was.
    pub(crate) fn leave_tape(&mut self) {
        let Some(kind) = self.shelf_kind() else {
            return;
        };
        let n = slot(kind);
        if self.media_path[n].pop().is_none() {
            return;
        }
        let at = self.media_back[n].pop().unwrap_or(0);
        self.fill_media(kind, at);
    }
}
