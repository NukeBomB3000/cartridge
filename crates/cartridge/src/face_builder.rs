use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use cartridge_store::Cart;
use cartridge_ui::{board_face, cart_face, padded, CartFace, TURN_PAD};

pub struct BuiltFaces {
    pub stem: String,
    pub board: CartFace,
    pub lid: CartFace,
}

pub struct FaceBuilder {
    requests: Sender<Cart>,
    built: Receiver<BuiltFaces>,
}

impl FaceBuilder {
    pub fn spawn() -> Self {
        let (requests, inbox) = mpsc::channel::<Cart>();
        let (outbox, built) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("cartridge-faces".into())
            .spawn(move || {
                while let Ok(mut cart) = inbox.recv() {
                    while let Ok(newer) = inbox.try_recv() {
                        cart = newer;
                    }
                    let faces = BuiltFaces {
                        stem: faces_key(&cart),
                        board: board_face(&cart),
                        lid: padded(&cart_face(&cart), TURN_PAD),
                    };
                    if outbox.send(faces).is_err() {
                        return;
                    }
                }
            });
        if let Err(e) = spawned {
            eprintln!("cartridge: faces: worker thread failed to start: {e}");
        }
        FaceBuilder { requests, built }
    }

    pub fn request(&self, cart: Cart) {
        let _ = self.requests.send(cart);
    }

    pub fn take(&self) -> Option<BuiltFaces> {
        let mut newest = None;
        while let Ok(faces) = self.built.try_recv() {
            newest = Some(faces);
        }
        newest
    }
}

/// What a cart's faces are known by: its system and its name, as two systems can each have a
/// game of the same name, 007 on the Game Boy Color and on the PlayStation alike.
pub fn faces_key(cart: &Cart) -> String {
    format!("{}/{}", cart.platform.dir_name(), cart.stem)
}
