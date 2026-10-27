//! The cart the shelf rests on, read through once into the page cache while nothing else is
//! going on, so the game starts without waiting on the SD card. A cart is read once it has been
//! rested on for `PREFETCH_REST_MS`, and the read stops the moment the shelf moves on. A disc
//! image too big to be worth the card's time is left alone. slot's.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use cartridge_input::Millis;

pub const PREFETCH_REST_MS: Millis = 500;

const CHUNK: usize = 1 << 20;

/// The biggest file read ahead: every cart's ROM, but not a PlayStation disc, hundreds of MB the
/// card would be kept busy with for little gain.
pub const PREFETCH_MAX_BYTES: u64 = 64 << 20;

#[derive(Default)]
pub struct Prefetch {
    resting: Option<(PathBuf, Millis)>,
    read: Option<PathBuf>,
    turn: Arc<AtomicU64>,
    worker: Option<JoinHandle<u64>>,
}

impl Prefetch {
    pub fn browse(&mut self, rom: Option<&Path>, now: Millis) {
        let Some(rom) = rom else {
            self.resting = None;
            self.read = None;
            self.turn.fetch_add(1, Ordering::Relaxed);
            return;
        };
        let since = match &self.resting {
            Some((at, since)) if at == rom => *since,
            _ => {
                self.resting = Some((rom.to_path_buf(), now));
                self.read = None;
                self.turn.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };
        if now.saturating_sub(since) < PREFETCH_REST_MS || self.read.as_deref() == Some(rom) {
            return;
        }
        if std::fs::metadata(rom).map_or(true, |m| m.len() > PREFETCH_MAX_BYTES) {
            return;
        }
        self.read = Some(rom.to_path_buf());
        let turn = self.turn.clone();
        let mine = turn.fetch_add(1, Ordering::Relaxed) + 1;
        let path = rom.to_path_buf();
        self.worker = std::thread::Builder::new()
            .name("cartridge-prefetch".into())
            .spawn(move || read_through(&path, &turn, mine))
            .ok();
    }

    pub fn reading(&self) -> Option<&Path> {
        self.read.as_deref()
    }

    pub fn wait(&mut self) -> Option<u64> {
        self.worker.take().and_then(|w| w.join().ok())
    }
}

fn read_through(path: &Path, turn: &AtomicU64, mine: u64) -> u64 {
    let start = Instant::now();
    let Ok(mut file) = std::fs::File::open(path) else {
        return 0;
    };
    let mut buf = vec![0u8; CHUNK];
    let mut total = 0u64;
    while turn.load(Ordering::Relaxed) == mine {
        match file.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => total += n as u64,
        }
    }
    eprintln!(
        "cartridge: prefetch: {} bytes of {} in {:.1} ms",
        total,
        path.display(),
        start.elapsed().as_secs_f64() * 1e3
    );
    total
}
