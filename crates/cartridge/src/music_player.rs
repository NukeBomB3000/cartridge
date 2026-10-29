//! The music player: the songs of a tape decoded on a thread of its own with symphonia, made
//! stereo and resampled to the ring's rate, and fed to the audio ring as it takes them. What the
//! screen shows (the song, how far into it, paused or not) the thread keeps in [`Shared`].

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, MetadataRevision, StandardTagKey};
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

use crate::audio::Ring;
use crate::resample::Resampler;

/// How far Left and Right jump within a song, in seconds.
pub const SEEK_SECS: i64 = 10;

/// How long the thread waits for room in the ring, or for a command while paused, in between.
const NAP: Duration = Duration::from_millis(4);

/// The share of the ring that is kept filled: sound is fed to it until it holds this much, so a
/// pause or a skip is heard soon.
const FILL: usize = 3;
const OF: usize = 4;

/// What the thread is told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    /// Play or pause.
    Toggle,
    Next,
    Prev,
    /// Jump this many seconds within the song, back if negative.
    Seek(i64),
    Stop,
}

/// What the thread tells the screen, read without waiting.
pub struct Shared {
    pub index: AtomicUsize,
    pub count: usize,
    pub pos_ms: AtomicU64,
    pub dur_ms: AtomicU64,
    pub paused: AtomicBool,
    /// The last song has played out.
    pub ended: AtomicBool,
    pub volume: AtomicU8,
    pub title: Mutex<String>,
}

pub struct Player {
    tx: Sender<Cmd>,
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

/// What the screen shows of the song that plays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub title: String,
    /// The song's place in the tape, from 1, and the number of songs.
    pub number: usize,
    pub count: usize,
    pub pos_ms: u64,
    pub dur_ms: u64,
    pub paused: bool,
    pub ended: bool,
}

impl Player {
    /// Starts playing `songs` into `ring`, at `volume`, from the song `start.0` and its
    /// millisecond `start.1`.
    pub fn start(ring: Arc<Ring>, songs: Vec<PathBuf>, volume: u8, start: (usize, u64)) -> Player {
        let (tx, rx) = channel();
        let shared = Arc::new(Shared {
            index: AtomicUsize::new(if start.0 < songs.len() { start.0 } else { 0 }),
            count: songs.len(),
            pos_ms: AtomicU64::new(if start.0 < songs.len() { start.1 } else { 0 }),
            dur_ms: AtomicU64::new(0),
            paused: AtomicBool::new(false),
            ended: AtomicBool::new(false),
            volume: AtomicU8::new(volume),
            title: Mutex::new(String::new()),
        });
        let thread = {
            let shared = shared.clone();
            std::thread::Builder::new()
                .name("cartridge-music".into())
                .spawn(move || run(&ring, &songs, &shared, &rx, start))
                .ok()
        };
        Player { tx, shared, thread }
    }

    pub fn send(&self, cmd: Cmd) {
        let _ = self.tx.send(cmd);
    }

    pub fn set_volume(&self, volume: u8) {
        self.shared.volume.store(volume.min(100), Ordering::Relaxed);
    }

    pub fn status(&self) -> Status {
        let s = &self.shared;
        Status {
            title: s.title.lock().map(|t| t.clone()).unwrap_or_default(),
            number: s.index.load(Ordering::Relaxed) + 1,
            count: s.count,
            pos_ms: s.pos_ms.load(Ordering::Relaxed),
            dur_ms: s.dur_ms.load(Ordering::Relaxed),
            paused: s.paused.load(Ordering::Relaxed),
            ended: s.ended.load(Ordering::Relaxed),
        }
    }

    /// Whether sound is coming out: playing, not paused and not played out.
    pub fn playing(&self) -> bool {
        !self.shared.paused.load(Ordering::Relaxed) && !self.shared.ended.load(Ordering::Relaxed)
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        let _ = self.tx.send(Cmd::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// A time as the screen prints it: minutes and seconds, `3:07`.
pub fn clock(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Interleaved `channels`-channel samples as stereo: a mono song's one channel on both sides, any
/// more than two cut to the first two.
pub fn stereo(interleaved: &[i16], channels: usize) -> Vec<i16> {
    match channels {
        0 => Vec::new(),
        1 => interleaved.iter().flat_map(|&s| [s, s]).collect(),
        2 => interleaved.to_vec(),
        n => interleaved
            .chunks_exact(n)
            .flat_map(|frame| [frame[0], frame[1]])
            .collect(),
    }
}

/// One song, opened for decoding.
pub struct Track {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    pub rate: u32,
    pub dur_ms: u64,
    pub title: String,
    /// Where in the song the last chunk ended, in milliseconds.
    pub pos_ms: u64,
    time_base: Option<symphonia::core::units::TimeBase>,
}

fn title_of(meta: Option<&MetadataRevision>) -> Option<String> {
    meta?
        .tags()
        .iter()
        .find(|t| t.std_key == Some(StandardTagKey::TrackTitle))
        .map(|t| t.value.to_string())
        .filter(|t| !t.trim().is_empty())
}

impl Track {
    pub fn open(path: &Path) -> Option<Track> {
        let file = File::open(path).ok()?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let mut probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .ok()?;
        let tagged = probed.metadata.get();
        let mut format = probed.format;
        let title = title_of(format.metadata().current())
            .or_else(|| title_of(tagged.as_ref().and_then(|m| m.current())))
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string()
            });
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)?;
        let params = track.codec_params.clone();
        let rate = params.sample_rate?;
        let dur_ms = params
            .n_frames
            .map_or(0, |n| n * 1000 / u64::from(rate.max(1)));
        let decoder = symphonia::default::get_codecs()
            .make(&params, &DecoderOptions::default())
            .ok()?;
        Some(Track {
            track_id: track.id,
            time_base: params.time_base,
            format,
            decoder,
            rate,
            dur_ms,
            title,
            pos_ms: 0,
        })
    }

    /// The next stretch of the song as stereo samples at its own rate, or None at its end.
    pub fn next_chunk(&mut self) -> Option<Vec<i16>> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(p) => p,
                Err(_) => return None,
            };
            if packet.track_id() != self.track_id {
                continue;
            }
            if let Some(tb) = self.time_base {
                let t = tb.calc_time(packet.ts());
                self.pos_ms = (t.seconds * 1000) + (t.frac * 1000.0) as u64;
            }
            match self.decoder.decode(&packet) {
                Ok(buf) => {
                    let spec = *buf.spec();
                    let mut samples = SampleBuffer::<i16>::new(buf.capacity() as u64, spec);
                    samples.copy_interleaved_ref(buf);
                    return Some(stereo(samples.samples(), spec.channels.count()));
                }
                // a bad frame is skipped, as a deck would play on
                Err(Error::DecodeError(_)) => continue,
                Err(_) => return None,
            }
        }
    }

    /// Jumps to `ms` in the song; false where the format cannot.
    pub fn seek_to(&mut self, ms: u64) -> bool {
        let time = Time::new(ms / 1000, (ms % 1000) as f64 / 1000.0);
        let to = SeekTo::Time {
            time,
            track_id: Some(self.track_id),
        };
        match self.format.seek(SeekMode::Coarse, to) {
            Ok(done) => {
                self.decoder.reset();
                self.pos_ms = match self.time_base {
                    Some(tb) => {
                        let t = tb.calc_time(done.actual_ts);
                        t.seconds * 1000 + (t.frac * 1000.0) as u64
                    }
                    None => ms,
                };
                true
            }
            Err(_) => false,
        }
    }
}

/// What the thread was told to do next, after a song or a pause.
enum Next {
    Stay,
    Go(usize),
    Stop,
}

fn run(ring: &Ring, songs: &[PathBuf], shared: &Shared, rx: &Receiver<Cmd>, start: (usize, u64)) {
    let mut at = if start.0 < songs.len() { start.0 } else { 0 };
    let mut resume = (start.0 < songs.len())
        .then_some(start.1)
        .filter(|ms| *ms > 0);
    while at < songs.len() {
        shared.index.store(at, Ordering::Relaxed);
        if resume.is_none() {
            shared.pos_ms.store(0, Ordering::Relaxed);
        }
        let Some(mut track) = Track::open(&songs[at]) else {
            at += 1;
            continue;
        };
        if let Ok(mut title) = shared.title.lock() {
            *title = track.title.clone();
        }
        shared.dur_ms.store(track.dur_ms, Ordering::Relaxed);
        // the first song goes on from where it was left
        if let Some(ms) = resume.take() {
            if track.dur_ms == 0 || ms + 5_000 < track.dur_ms {
                track.seek_to(ms);
                shared.pos_ms.store(track.pos_ms, Ordering::Relaxed);
            }
        }
        match play_track(ring, &mut track, shared, rx, at, songs.len()) {
            Next::Stop => return,
            Next::Go(to) => at = to,
            Next::Stay => at += 1,
        }
    }
    shared.ended.store(true, Ordering::Relaxed);
}

fn play_track(
    ring: &Ring,
    track: &mut Track,
    shared: &Shared,
    rx: &Receiver<Cmd>,
    at: usize,
    count: usize,
) -> Next {
    let mut resampler = None;
    let mut rate = 0;
    let mut out = Vec::new();
    loop {
        // commands first, also while paused
        loop {
            match rx.try_recv() {
                Ok(Cmd::Stop) | Err(TryRecvError::Disconnected) => return Next::Stop,
                Ok(Cmd::Toggle) => {
                    let paused = shared.paused.load(Ordering::Relaxed);
                    shared.paused.store(!paused, Ordering::Relaxed);
                }
                Ok(Cmd::Next) => return Next::Go((at + 1).min(count)),
                Ok(Cmd::Prev) => {
                    // a song more than a few seconds in starts again; near its start, the one before
                    return if track.pos_ms > 3000 {
                        Next::Go(at)
                    } else {
                        Next::Go(at.saturating_sub(1))
                    };
                }
                Ok(Cmd::Seek(secs)) => {
                    let to = (track.pos_ms as i64 + secs * 1000).max(0) as u64;
                    if track.dur_ms > 0 && to >= track.dur_ms {
                        return Next::Stay;
                    }
                    track.seek_to(to);
                    resampler = None;
                    shared.pos_ms.store(track.pos_ms, Ordering::Relaxed);
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        if shared.paused.load(Ordering::Relaxed) {
            std::thread::sleep(NAP * 5);
            continue;
        }
        let out_rate = ring.sample_rate();
        if out_rate == 0 {
            std::thread::sleep(NAP * 5);
            continue;
        }
        // keep the ring filled to the share, no more
        let room = ring.capacity_frames() * FILL / OF;
        if ring.queued_frames() >= room {
            std::thread::sleep(NAP);
            continue;
        }
        let Some(chunk) = track.next_chunk() else {
            return Next::Stay;
        };
        shared.pos_ms.store(track.pos_ms, Ordering::Relaxed);
        if resampler.is_none() || rate != out_rate {
            rate = out_rate;
            resampler = Some(Resampler::new(f64::from(track.rate), f64::from(out_rate)));
        }
        let mut samples = if track.rate == out_rate {
            chunk
        } else {
            let Some(r) = resampler.as_mut() else {
                continue;
            };
            r.process(&chunk, &mut out);
            std::mem::take(&mut out)
        };
        crate::audio::volume::apply(&mut samples, shared.volume.load(Ordering::Relaxed));
        ring.push(&samples);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A WAV of a 440 Hz tone, 16-bit, `secs` long at `rate`, `channels` wide.
    fn wav(rate: u32, channels: u16, secs: f32) -> Vec<u8> {
        let frames = (rate as f32 * secs) as usize;
        let mut data = Vec::new();
        for i in 0..frames {
            let s =
                ((i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 12000.0) as i16;
            for _ in 0..channels {
                data.extend_from_slice(&s.to_le_bytes());
            }
        }
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&channels.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
        out.extend_from_slice(&(channels * 2).to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&data);
        out
    }

    #[test]
    fn a_time_is_minutes_and_seconds() {
        assert_eq!(clock(0), "0:00");
        assert_eq!(clock(187_000), "3:07");
        assert_eq!(clock(61_999), "1:01");
    }

    #[test]
    fn every_song_comes_out_stereo() {
        assert_eq!(stereo(&[1, 2, 3], 1), [1, 1, 2, 2, 3, 3]);
        assert_eq!(stereo(&[1, 2, 3, 4], 2), [1, 2, 3, 4]);
        assert_eq!(stereo(&[1, 2, 9, 3, 4, 9], 3), [1, 2, 3, 4]);
        assert!(stereo(&[1, 2], 0).is_empty());
    }

    #[test]
    fn a_track_is_decoded_to_the_end_with_its_rate_and_length() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("01 Tone.wav");
        std::fs::write(&path, wav(8000, 1, 0.5)).unwrap();
        let mut track = Track::open(&path).expect("a wav opens");
        assert_eq!(track.rate, 8000);
        assert_eq!(track.title, "01 Tone");
        assert!((track.dur_ms as i64 - 500).abs() < 5, "{}", track.dur_ms);
        let mut frames = 0;
        while let Some(chunk) = track.next_chunk() {
            assert_eq!(chunk.len() % 2, 0, "stereo");
            frames += chunk.len() / 2;
        }
        assert_eq!(frames, 4000);
    }

    #[test]
    fn a_file_that_is_no_song_does_not_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.mp3");
        std::fs::write(&path, b"not music at all").unwrap();
        assert!(Track::open(&path).is_none());
    }

    #[test]
    fn the_player_feeds_the_ring_pauses_and_ends() {
        let dir = tempfile::tempdir().unwrap();
        let songs: Vec<PathBuf> = (1..=2)
            .map(|i| {
                let path = dir.path().join(format!("{i}.wav"));
                std::fs::write(&path, wav(44100, 2, 0.2)).unwrap();
                path
            })
            .collect();
        let ring = Arc::new(Ring::new(crate::audio::ring_capacity(48000)));
        ring.reopen(48000);
        let player = Player::start(ring.clone(), songs, 100, (0, 0));
        let wait = |what: &str, mut done: Box<dyn FnMut() -> bool>| {
            for _ in 0..400 {
                if done() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            panic!("timed out waiting for {what}");
        };
        wait(
            "sound in the ring",
            Box::new(|| ring.queued_frames() > 1000),
        );
        assert_eq!(player.status().count, 2);
        // a second press of play is a pause, and the third plays on
        player.send(Cmd::Toggle);
        wait("the pause", Box::new(|| player.status().paused));
        assert!(!player.playing());
        player.send(Cmd::Toggle);
        wait("playing again", Box::new(|| !player.status().paused));
        // drain what is queued so the thread can finish both songs
        let mut buf = vec![0i16; 2048];
        wait(
            "the end",
            Box::new(|| {
                ring.fill(&mut buf);
                player.status().ended
            }),
        );
        assert_eq!(player.status().number, 2);
    }
}
