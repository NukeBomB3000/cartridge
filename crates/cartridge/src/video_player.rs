//! The video player. A video is decoded by FFmpeg, run as two child processes: one decodes the
//! picture, scaled to the screen and written as XRGB8888 frames, the other the sound, 48 kHz
//! stereo, each to a pipe. The sound is the clock: it is fed to the audio ring as the ring takes
//! it, the picture is shown when the sound has got as far as the frame, and a frame the sound has
//! already passed is dropped. A video with no sound runs by the wall clock.
//!
//! `System/ffmpeg` is a build of FFmpeg's own source, LGPL, with only the decoders the player
//! needs: it is a program run on its own, not linked in.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::audio::Ring;
use crate::frames::{FrameRef, Frames};

/// How far Left and Right jump within a video, in seconds.
pub const SEEK_SECS: i64 = 10;
/// How far Up and Down jump.
pub const SEEK_BIG_SECS: i64 = 60;

/// How late a frame may be, in milliseconds, before it is dropped rather than shown.
const LATE_MS: i64 = 120;
/// The sound's rate and the size of a chunk of it fed to the ring, in frames.
const HZ: u32 = 48_000;
const CHUNK: usize = 2048;
const NAP: Duration = Duration::from_millis(3);

/// What the player is asked to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    Toggle,
    /// Jump this many seconds, back if negative.
    Seek(i64),
    /// The next video of the folder, or the one before.
    Next,
    Prev,
    /// The next sound track of the video.
    Track,
    Stop,
}

/// What `ffmpeg -i` says of a video.
#[derive(Clone, Debug, PartialEq)]
pub struct Info {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub duration_ms: u64,
    /// The sound tracks, each as the language it is tagged with, empty if untagged.
    pub audio: Vec<String>,
}

/// `Duration: 01:30:08.89,` as milliseconds.
fn parse_duration(line: &str) -> Option<u64> {
    let rest = line.split("Duration:").nth(1)?.trim();
    let time = rest.split(',').next()?.trim();
    let mut parts = time.split(':');
    let (h, m, s) = (parts.next()?, parts.next()?, parts.next()?);
    let secs: f64 = s.parse().ok()?;
    let ms = (h.parse::<u64>().ok()? * 3600 + m.parse::<u64>().ok()? * 60) * 1000;
    Some(ms + (secs * 1000.0) as u64)
}

/// The size, shape and frame rate of a `Video:` stream line, as its picture is meant to be seen: a
/// stream with a pixel that is not square is as wide as its pixels make it.
fn parse_video(line: &str) -> Option<(u32, u32, f32)> {
    let body = line.split("Video:").nth(1)?;
    // the size is the first `WxH` after the codec's own words, `1920x1080 [SAR 1:1 ...]`
    let (mut w, mut h) = (0u32, 0u32);
    for word in body.split([' ', ',']) {
        let word = word.trim();
        if let Some((a, b)) = word.split_once('x') {
            if let (Ok(a), Ok(b)) = (a.parse::<u32>(), b.parse::<u32>()) {
                if a >= 16 && b >= 16 {
                    (w, h) = (a, b);
                    break;
                }
            }
        }
    }
    if w == 0 {
        return None;
    }
    if let Some(sar) = body.split("SAR ").nth(1) {
        let sar = sar.split_whitespace().next().unwrap_or("1:1");
        if let Some((n, d)) = sar.split_once(':') {
            if let (Ok(n), Ok(d)) = (n.parse::<f32>(), d.parse::<f32>()) {
                if n > 0.0 && d > 0.0 {
                    w = (w as f32 * n / d).round() as u32;
                }
            }
        }
    }
    let fps = body
        .split(',')
        .find_map(|part| {
            part.trim()
                .strip_suffix(" fps")
                .and_then(|f| f.parse::<f32>().ok())
        })
        .or_else(|| {
            body.split(',').find_map(|part| {
                part.trim()
                    .strip_suffix(" tbr")
                    .and_then(|f| f.parse::<f32>().ok())
            })
        })
        .filter(|f| *f > 1.0 && *f < 241.0)
        .unwrap_or(25.0);
    Some((w, h, fps))
}

/// The language a `Stream #0:1[0x2](eng): Audio:` line is tagged with.
fn parse_audio_language(line: &str) -> String {
    let head = line.split("Audio:").next().unwrap_or_default();
    match (head.rfind('('), head.rfind(')')) {
        (Some(a), Some(b)) if b > a && head[a + 1..b].chars().all(|c| c.is_ascii_alphabetic()) => {
            head[a + 1..b].to_string()
        }
        _ => String::new(),
    }
}

/// What the text `ffmpeg -i` prints says of the video.
pub fn parse_info(text: &str) -> Option<Info> {
    let mut duration_ms = 0;
    let mut video = None;
    let mut audio = Vec::new();
    for line in text.lines() {
        if line.contains("Duration:") && duration_ms == 0 {
            duration_ms = parse_duration(line).unwrap_or(0);
        } else if line.contains("Stream #") && line.contains("Video:") && video.is_none() {
            // a cover picture in an mp4 is a video stream of one frame: not the video
            if !line.contains("attached pic") {
                video = parse_video(line);
            }
        } else if line.contains("Stream #") && line.contains("Audio:") {
            audio.push(parse_audio_language(line));
        }
    }
    let (width, height, fps) = video?;
    Some(Info {
        width,
        height,
        fps,
        duration_ms,
        audio,
    })
}

/// Runs `ffmpeg -i file` for what it says of it.
pub fn probe(ffmpeg: &Path, file: &Path) -> Option<Info> {
    let out = Command::new(ffmpeg)
        .args(["-hide_banner", "-i"])
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .ok()?;
    parse_info(&String::from_utf8_lossy(&out.stderr))
}

/// The size a video of `info`'s shape is decoded to so as to fill `room` by `room` and no more,
/// whole numbers each, even, as the converter needs them.
pub fn fit(info: &Info, room: (u32, u32)) -> (u32, u32) {
    let scale = (room.0 as f32 / info.width as f32)
        .min(room.1 as f32 / info.height as f32)
        .min(1.0);
    let even = |v: f32| (((v / 2.0).round() as u32) * 2).max(2);
    (
        even(info.width as f32 * scale),
        even(info.height as f32 * scale),
    )
}

/// What the screen shows of the video, read without waiting.
pub struct Shared {
    pub pos_ms: AtomicU64,
    pub dur_ms: AtomicU64,
    pub paused: AtomicBool,
    pub ended: AtomicBool,
    pub volume: AtomicU8,
    pub index: AtomicUsize,
    pub count: usize,
    pub track: AtomicUsize,
    pub tracks: AtomicUsize,
    pub title: Mutex<String>,
    /// The file that plays.
    pub file: Mutex<PathBuf>,
    pub published: AtomicBool,
}

/// What the screen shows of the video, copied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub title: String,
    /// The file that plays.
    pub file: PathBuf,
    pub number: usize,
    pub count: usize,
    pub pos_ms: u64,
    pub dur_ms: u64,
    pub paused: bool,
    pub ended: bool,
    /// The sound track playing, from 1, and how many there are.
    pub track: usize,
    pub tracks: usize,
}

pub struct Player {
    tx: Sender<Cmd>,
    shared: Arc<Shared>,
    frames: Arc<Frames>,
    thread: Option<JoinHandle<()>>,
}

impl Player {
    /// Plays `files` from the first, in `room` pixels, its sound into `ring` at `volume`.
    pub fn start(
        ffmpeg: PathBuf,
        ring: Arc<Ring>,
        files: Vec<PathBuf>,
        room: (u32, u32),
        volume: u8,
        start_ms: Option<u64>,
    ) -> Player {
        let (tx, rx) = channel();
        let shared = Arc::new(Shared {
            pos_ms: AtomicU64::new(0),
            dur_ms: AtomicU64::new(0),
            paused: AtomicBool::new(false),
            ended: AtomicBool::new(false),
            volume: AtomicU8::new(volume),
            index: AtomicUsize::new(0),
            count: files.len(),
            track: AtomicUsize::new(0),
            tracks: AtomicUsize::new(0),
            title: Mutex::new(String::new()),
            file: Mutex::new(PathBuf::new()),
            published: AtomicBool::new(false),
        });
        let frames = Frames::new((room.0 * room.1 * 4) as usize);
        let thread = {
            let (shared, frames) = (shared.clone(), frames.clone());
            std::thread::Builder::new()
                .name("cartridge-video".into())
                .spawn(move || {
                    run(
                        &ffmpeg, &ring, &files, room, &shared, &frames, &rx, start_ms,
                    )
                })
                .ok()
        };
        Player {
            tx,
            shared,
            frames,
            thread,
        }
    }

    pub fn send(&self, cmd: Cmd) {
        let _ = self.tx.send(cmd);
    }

    pub fn set_volume(&self, volume: u8) {
        self.shared.volume.store(volume.min(100), Ordering::Relaxed);
    }

    /// The newest picture, if one is waiting.
    pub fn frame(&self) -> Option<FrameRef> {
        self.frames.latest()
    }

    pub fn published(&self) -> bool {
        self.shared.published.load(Ordering::Relaxed)
    }

    pub fn status(&self) -> Status {
        let s = &self.shared;
        Status {
            title: s.title.lock().map(|t| t.clone()).unwrap_or_default(),
            file: s.file.lock().map(|f| f.clone()).unwrap_or_default(),
            number: s.index.load(Ordering::Relaxed) + 1,
            count: s.count,
            pos_ms: s.pos_ms.load(Ordering::Relaxed),
            dur_ms: s.dur_ms.load(Ordering::Relaxed),
            paused: s.paused.load(Ordering::Relaxed),
            ended: s.ended.load(Ordering::Relaxed),
            track: s.track.load(Ordering::Relaxed) + 1,
            tracks: s.tracks.load(Ordering::Relaxed),
        }
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

/// The two child processes of one video from one place in it, and what they were started for.
struct Run {
    video: Child,
    audio: Option<Child>,
}

impl Run {
    fn stop(mut self) {
        let _ = self.video.kill();
        let _ = self.video.wait();
        if let Some(mut audio) = self.audio.take() {
            let _ = audio.kill();
            let _ = audio.wait();
        }
    }
}

fn spawn(
    ffmpeg: &Path,
    file: &Path,
    from_ms: u64,
    size: (u32, u32),
    track: Option<usize>,
) -> Option<(Run, ChildStdout, Option<ChildStdout>)> {
    let secs = format!("{:.3}", from_ms as f64 / 1000.0);
    let mut video = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-ss", &secs, "-i"])
        .arg(file)
        .args(["-map", "0:v:0", "-an", "-sn", "-threads", "4", "-vf"])
        .arg(format!(
            "scale={}:{}:flags=fast_bilinear,format=bgra",
            size.0, size.1
        ))
        .args(["-f", "rawvideo", "pipe:1"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let video_out = video.stdout.take()?;
    let (audio, audio_out) = match track {
        Some(n) => {
            let mut child = Command::new(ffmpeg)
                .args(["-hide_banner", "-loglevel", "error", "-ss", &secs, "-i"])
                .arg(file)
                .args(["-map", &format!("0:a:{n}"), "-vn", "-sn", "-f", "s16le"])
                .args(["-ar", &HZ.to_string(), "-ac", "2", "pipe:1"])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            let out = child.stdout.take();
            (Some(child), out)
        }
        None => (None, None),
    };
    Some((Run { video, audio }, video_out, audio_out))
}

/// Reads exactly `buf.len()` bytes from a pipe, false at its end.
fn read_full(pipe: &mut impl Read, buf: &mut [u8]) -> bool {
    let mut at = 0;
    while at < buf.len() {
        match pipe.read(&mut buf[at..]) {
            Ok(0) | Err(_) => return false,
            Ok(n) => at += n,
        }
    }
    true
}

/// The clock a video is shown by: the sound as it has played out, else the wall.
struct Clock {
    from_ms: u64,
    /// Sound frames given to the ring since the run began.
    fed: Arc<AtomicU64>,
    ring: Arc<Ring>,
    audio: bool,
    started: Instant,
    paused_at: Option<Instant>,
    paused_total: Duration,
}

impl Clock {
    /// Where in the video the picture should be, in milliseconds.
    fn now_ms(&self) -> i64 {
        let played = if self.audio {
            let fed = self.fed.load(Ordering::Relaxed);
            fed.saturating_sub(self.ring.queued_frames() as u64) * 1000 / u64::from(HZ)
        } else {
            let held = self.paused_at.map_or(Duration::ZERO, |at| at.elapsed());
            (self
                .started
                .elapsed()
                .saturating_sub(self.paused_total + held))
            .as_millis() as u64
        };
        (self.from_ms + played) as i64
    }

    fn pause(&mut self, on: bool) {
        match (on, self.paused_at) {
            (true, None) => self.paused_at = Some(Instant::now()),
            (false, Some(at)) => {
                self.paused_total += at.elapsed();
                self.paused_at = None;
            }
            _ => {}
        }
    }
}

enum Next {
    /// The video played out: the next one.
    Over,
    Go(usize),
    Stop,
}

/// How a stretch of playing ended: to go on from another place or track, or for good.
enum Out {
    Restart,
    Done(Next),
}

fn run(
    ffmpeg: &Path,
    ring: &Arc<Ring>,
    files: &[PathBuf],
    room: (u32, u32),
    shared: &Arc<Shared>,
    frames: &Arc<Frames>,
    rx: &Receiver<Cmd>,
    start_ms: Option<u64>,
) {
    let mut at = 0usize;
    let mut first = start_ms;
    while at < files.len() {
        shared.index.store(at, Ordering::Relaxed);
        shared.pos_ms.store(0, Ordering::Relaxed);
        shared.track.store(0, Ordering::Relaxed);
        let file = &files[at];
        let title = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if let Ok(mut t) = shared.title.lock() {
            *t = title;
        }
        if let Ok(mut f) = shared.file.lock() {
            *f = file.clone();
        }
        let Some(info) = probe(ffmpeg, file) else {
            at += 1;
            continue;
        };
        shared.dur_ms.store(info.duration_ms, Ordering::Relaxed);
        shared.tracks.store(info.audio.len(), Ordering::Relaxed);
        // the first video goes on from where it was left, if that is worth it
        let from = crate::media_resume::video_start(first.take(), info.duration_ms);
        let video = Playing {
            ffmpeg,
            ring,
            file,
            info: &info,
            room,
            shared,
            frames,
            rx,
            at,
            count: files.len(),
            from,
        };
        match video.play() {
            Next::Stop => return,
            Next::Go(to) => at = to,
            Next::Over => at += 1,
        }
    }
    shared.ended.store(true, Ordering::Relaxed);
}

/// One video being played.
struct Playing<'a> {
    ffmpeg: &'a Path,
    ring: &'a Arc<Ring>,
    file: &'a Path,
    info: &'a Info,
    room: (u32, u32),
    shared: &'a Arc<Shared>,
    frames: &'a Arc<Frames>,
    rx: &'a Receiver<Cmd>,
    at: usize,
    count: usize,
    /// Where the video begins, in milliseconds.
    from: u64,
}

impl Playing<'_> {
    fn play(&self) -> Next {
        let (shared, info) = (self.shared, self.info);
        let size = fit(info, self.room);
        let frame_bytes = (size.0 * size.1 * 4) as usize;
        let frame_ms = 1000.0 / f64::from(info.fps);
        let mut from_ms = self.from;
        let mut track = 0usize;
        loop {
            let sound = (!info.audio.is_empty()).then_some(track);
            let Some((run, mut video_out, audio_out)) =
                spawn(self.ffmpeg, self.file, from_ms, size, sound)
            else {
                return Next::Over;
            };
            let fed = Arc::new(AtomicU64::new(0));
            let stop = Arc::new(AtomicBool::new(false));
            // the sound on a thread of its own, feeding the ring as it takes it
            let audio_thread = audio_out.map(|mut pipe| {
                let (fed, stop, ring, shared) =
                    (fed.clone(), stop.clone(), self.ring.clone(), shared.clone());
                std::thread::spawn(move || feed_audio(&mut pipe, &ring, &fed, &stop, &shared))
            });
            let mut clock = Clock {
                from_ms,
                fed: fed.clone(),
                ring: self.ring.clone(),
                audio: audio_thread.is_some(),
                started: Instant::now(),
                paused_at: None,
                paused_total: Duration::ZERO,
            };
            let mut buf = self.frames.take_write();
            buf.resize(frame_bytes, 0);
            let mut index = 0u64;
            let mut dropped = 0u64;
            let trace = std::env::var_os("CARTRIDGE_TRACE").is_some();
            let out = 'session: loop {
                // what the player is asked to do
                let mut restart = false;
                loop {
                    match self.rx.try_recv() {
                        Ok(Cmd::Stop) | Err(TryRecvError::Disconnected) => {
                            break 'session Out::Done(Next::Stop)
                        }
                        Ok(Cmd::Toggle) => {
                            let on = !shared.paused.load(Ordering::Relaxed);
                            shared.paused.store(on, Ordering::Relaxed);
                            clock.pause(on);
                        }
                        Ok(Cmd::Next) => {
                            break 'session Out::Done(Next::Go((self.at + 1).min(self.count)))
                        }
                        Ok(Cmd::Prev) => {
                            // a video more than a few seconds in starts again
                            let to = if clock.now_ms() > 3000 {
                                self.at
                            } else {
                                self.at.saturating_sub(1)
                            };
                            break 'session Out::Done(Next::Go(to));
                        }
                        Ok(Cmd::Seek(secs)) => {
                            from_ms = (clock.now_ms().max(0) + secs * 1000).max(0) as u64;
                            if info.duration_ms > 0 && from_ms + 1000 >= info.duration_ms {
                                break 'session Out::Done(Next::Over);
                            }
                            restart = true;
                        }
                        Ok(Cmd::Track) => {
                            if info.audio.len() > 1 {
                                track = (track + 1) % info.audio.len();
                                shared.track.store(track, Ordering::Relaxed);
                                from_ms = clock.now_ms().max(0) as u64;
                                restart = true;
                            }
                        }
                        Err(TryRecvError::Empty) => break,
                    }
                }
                if restart {
                    break Out::Restart;
                }
                // paused: the sound waits, and the picture with it
                if shared.paused.load(Ordering::Relaxed) {
                    std::thread::sleep(NAP * 4);
                    continue;
                }
                if !read_full(&mut video_out, &mut buf) {
                    break Out::Done(Next::Over);
                }
                let due = from_ms as f64 + index as f64 * frame_ms;
                index += 1;
                // wait for the sound to reach the frame
                while !shared.paused.load(Ordering::Relaxed) && (clock.now_ms() as f64) + 2.0 < due
                {
                    std::thread::sleep(NAP);
                }
                let now = clock.now_ms();
                shared.pos_ms.store(now.max(0) as u64, Ordering::Relaxed);
                // a frame the sound is long past is dropped, to catch up
                if trace && index % 150 == 0 {
                    eprintln!(
                        "cartridge: video: frame {index}, dropped {dropped}, {} ms from its time",
                        now - due as i64
                    );
                }
                if now - due as i64 > LATE_MS {
                    dropped += 1;
                    continue;
                }
                let ready = std::mem::replace(&mut buf, self.frames.take_write());
                buf.resize(frame_bytes, 0);
                self.frames.publish(ready, size);
                shared.published.store(true, Ordering::Relaxed);
            };
            stop.store(true, Ordering::Relaxed);
            run.stop();
            if let Some(t) = audio_thread {
                let _ = t.join();
            }
            match out {
                Out::Restart => continue,
                Out::Done(next) => return next,
            }
        }
    }
}

/// Feeds the sound read from `pipe` to `ring` as the ring has room for it, counting the frames
/// given, until the pipe ends or `stop` is set. While the video is paused it waits, the pipe
/// unread, so the decoder waits too.
fn feed_audio(
    pipe: &mut impl Read,
    ring: &Ring,
    fed: &AtomicU64,
    stop: &AtomicBool,
    shared: &Shared,
) {
    let mut bytes = vec![0u8; CHUNK * 4];
    while !stop.load(Ordering::Relaxed) {
        if shared.paused.load(Ordering::Relaxed)
            || ring.sample_rate() == 0
            || ring.queued_frames() >= ring.capacity_frames() * 3 / 4
        {
            std::thread::sleep(NAP);
            continue;
        }
        if !read_full(pipe, &mut bytes) {
            return;
        }
        let mut samples: Vec<i16> = bytes
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();
        crate::audio::volume::apply(&mut samples, shared.volume.load(Ordering::Relaxed));
        ring.push(&samples);
        fed.fetch_add(CHUNK as u64, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWIN_PEAKS: &str = "  Duration: 01:30:08.89, start: 0.000000, bitrate: 1606 kb/s
  Stream #0:0[0x1](und): Video: h264 (Main) (avc1 / 0x31637661), yuv420p(tv, bt470bg, progressive), 720x406 [SAR 1:1 DAR 360:203], 1481 kb/s, 25 fps, 25 tbr, 25k tbn (default)
  Stream #0:1[0x2](ger): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 125 kb/s (default)
  Stream #0:2[0x3](eng): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 125 kb/s";

    #[test]
    fn what_ffmpeg_says_of_a_video_is_read() {
        let info = parse_info(TWIN_PEAKS).expect("a video");
        assert_eq!((info.width, info.height), (720, 406));
        assert_eq!(info.fps, 25.0);
        assert_eq!(info.duration_ms, 5_408_890);
        assert_eq!(info.audio, ["ger", "eng"]);
    }

    #[test]
    fn a_pixel_that_is_not_square_makes_the_picture_wider() {
        let line = "  Stream #0:0: Video: mpeg2video, yuv420p, 720x576 [SAR 16:15 DAR 4:3], 25 fps, 25 tbr";
        let info = parse_info(&format!(
            "  Duration: 00:00:10.00, start: 0
{line}"
        ))
        .unwrap();
        assert_eq!((info.width, info.height), (768, 576));
    }

    #[test]
    fn a_cover_picture_is_not_the_video_and_a_file_without_a_video_is_none() {
        let text = "  Duration: 00:03:00.00, start: 0
  Stream #0:0: Audio: mp3, 44100 Hz, stereo
  Stream #0:1: Video: mjpeg, yuvj420p, 500x500, 90k tbr (attached pic)";
        assert!(parse_info(text).is_none());
        assert!(parse_info("nothing here").is_none());
    }

    #[test]
    fn the_picture_is_decoded_no_bigger_than_the_screen_and_never_enlarged() {
        let info = parse_info(TWIN_PEAKS).unwrap();
        assert_eq!(fit(&info, (960, 540)), (720, 406));
        assert_eq!(fit(&info, (640, 480)), (640, 360));
        let big = Info {
            width: 1920,
            height: 1080,
            fps: 30.0,
            duration_ms: 1,
            audio: vec![],
        };
        assert_eq!(fit(&big, (640, 480)), (640, 360));
        assert_eq!(fit(&big, (960, 540)), (960, 540));
    }
}
