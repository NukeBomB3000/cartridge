//! The threads that draw carts' faces from their labels, so no label is decoded on the main
//! thread: at boot for every cart that wears a scan, and for each label a sync saves. The shelf
//! comes up with every cart in its generated face, and each one is drawn again in its label as
//! its face comes back. A scan with no cut yet is cut first, so its face is drawn from the cut,
//! and once its face is handed back the scan is shrunk to what its faces need.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use cartridge_store::{Cart, Platform};
use cartridge_ui::{cart_face, cut_label, shrink_label, CartFace};

/// Drawing a face from its label is mostly decoding it, on one core; the rest are left to the
/// shelf.
const MAX_WORKERS: usize = 3;

/// How many drawn faces can wait for compose to take them. A worker with one more waits for
/// room, so a boot's hundreds of faces are never all held at once on a handheld with 1 GB.
const READY_FACES: usize = 8;

/// How far the workers stand back from the shelf, the game and its sound, as a nice value: a
/// boot that resumes a game draws its labels while the game runs, and the labels can wait.
#[cfg(feature = "device")]
const NICE: i32 = 10;

#[cfg(feature = "device")]
extern "C" {
    fn setpriority(which: i32, who: u32, prio: i32) -> i32;
}

/// Lets the calling thread run only on what the rest of cartridge leaves of the cores. Linux
/// keeps a nice value for each thread, and `setpriority` for process 0 sets the caller's.
fn stand_back() {
    #[cfg(feature = "device")]
    // SAFETY: setpriority only sets the calling thread's nice value; a refusal answers -1, and
    // the worker then runs as it would have
    unsafe {
        setpriority(0, 0, NICE);
    }
}

/// A cart to draw from its label, with the ticket it was asked under.
struct Job {
    cart: Cart,
    ticket: u64,
}

/// A cart's face drawn from its label, for the cart's face on the shelf.
pub struct Drawn {
    pub platform: Platform,
    pub stem: String,
    pub face: CartFace,
}

pub struct LabelCutter {
    requests: Sender<Job>,
    drawn: Receiver<(u64, Drawn)>,
    /// The ticket of the last face asked for each cart whose face has not come back yet: a face
    /// drawn for an ask a newer one has overtaken is not drawn over the newer one.
    asked: HashMap<(u8, String), u64>,
    tickets: u64,
}

impl LabelCutter {
    pub fn spawn() -> Self {
        let (requests, inbox) = mpsc::channel::<Job>();
        let (outbox, drawn) = mpsc::sync_channel(READY_FACES);
        let inbox = Arc::new(Mutex::new(inbox));
        let workers = thread::available_parallelism()
            .map_or(1, |n| n.get().saturating_sub(1))
            .clamp(1, MAX_WORKERS);
        for i in 0..workers {
            let inbox = Arc::clone(&inbox);
            let outbox = outbox.clone();
            let spawned = thread::Builder::new()
                .name(format!("cartridge-cut-{i}"))
                .spawn(move || {
                    stand_back();
                    loop {
                        let next = inbox.lock().map(|rx| rx.recv());
                        let Ok(Ok(Job { cart, ticket })) = next else {
                            return;
                        };
                        cut_label(&cart);
                        let face = Drawn {
                            platform: cart.platform,
                            stem: cart.stem.clone(),
                            face: cart_face(&cart),
                        };
                        if outbox.send((ticket, face)).is_err() {
                            return;
                        }
                        shrink_label(&cart);
                    }
                });
            if let Err(e) = spawned {
                eprintln!("cartridge: labels: cutter thread failed to start: {e}");
            }
        }
        LabelCutter {
            requests,
            drawn,
            asked: HashMap::new(),
            tickets: 0,
        }
    }

    /// Draws `cart`'s face from its label, after the carts asked for before it: its scans are
    /// cut first where they have no cut, and shrunk once its face is handed back.
    pub fn request(&mut self, cart: Cart) {
        self.tickets += 1;
        let key = (cart.platform as u8, cart.stem.clone());
        self.asked.insert(key, self.tickets);
        let _ = self.requests.send(Job {
            cart,
            ticket: self.tickets,
        });
    }

    /// The faces handed back and not taken yet, until `bytes` of them are taken, the rest left
    /// for the next time: always one at least, when one is waiting. A face drawn for a cart
    /// asked for again since is passed over.
    pub fn take(&mut self, bytes: usize) -> Vec<Drawn> {
        let mut taken = Vec::new();
        let mut spent = 0;
        while spent < bytes {
            let Ok((ticket, drawn)) = self.drawn.try_recv() else {
                break;
            };
            let key = (drawn.platform as u8, drawn.stem.clone());
            if self.asked.get(&key) != Some(&ticket) {
                continue;
            }
            self.asked.remove(&key);
            spent += drawn.face.rgba.len();
            taken.push(drawn);
        }
        taken
    }

    /// Whether every face asked for has come back and been taken.
    pub fn idle(&self) -> bool {
        self.asked.is_empty()
    }
}
