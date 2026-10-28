//! Runs a libretro core on a game with no picture or sound, pressing Start and A now and then
//! to get past title screens, and says how fast it ran: for judging a core on the device before
//! it has a shelf.
//!
//!   bench CORE ROM [FRAMES] [OUT.ppm]
//!
//! BENCH_OPTIONS="key=value,key=value" sets core options before the game loads, and BENCH_STATE
//! names a save state to put in once it has, to time a scene rather than a title screen.
//! BENCH_SAVE names a file the core's state is written to once the frames have run, for a scene to
//! time again with BENCH_STATE.
//! BENCH_PIN=1 puts this thread alone on the last core and every other thread of the process, the
//! core's own among them, on the first three, as DSperate's thread layout does, to see what that
//! is worth to a core that has no layout of its own.

use std::path::Path;
use std::time::Instant;

use cartridge_retro::{ButtonMask, LibretroCore, RetroCore};

extern "C" {
    fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
}

/// Puts thread `tid` on the cores in `mask`.
fn pin(tid: i32, mask: u64) {
    // SAFETY: the mask is one word, and a refusal only answers -1
    unsafe { sched_setaffinity(tid, 8, &mask) };
}

/// Puts this thread on core 3 and every other thread of the process on cores 0 to 2.
fn pin_threads() {
    let me = std::fs::read_to_string("/proc/thread-self/stat")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|t| t.parse::<i32>().ok())
        })
        .unwrap_or(0);
    pin(me, 1 << 3);
    if let Ok(dir) = std::fs::read_dir("/proc/self/task") {
        for entry in dir.flatten() {
            if let Some(tid) = entry
                .file_name()
                .to_str()
                .and_then(|t| t.parse::<i32>().ok())
            {
                if tid != me {
                    pin(tid, 0b0111);
                }
            }
        }
    }
}

const START: u16 = 1 << 3;
const A: u16 = 1 << 8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: bench CORE ROM [FRAMES] [OUT.ppm]");
        std::process::exit(2);
    }
    let frames: u32 = args.get(3).and_then(|f| f.parse().ok()).unwrap_or(1200);
    let mut core = LibretroCore::open_with(
        Path::new(&args[1]),
        Path::new("/mnt/sdcard/BIOS"),
        Path::new("/tmp"),
    )
    .unwrap_or_else(|e| panic!("open: {e}"));
    for pair in std::env::var("BENCH_OPTIONS")
        .unwrap_or_default()
        .split(',')
    {
        if let Some((key, value)) = pair.split_once('=') {
            core.set_option(key, value);
        }
    }
    core.load(Path::new(&args[2]))
        .unwrap_or_else(|e| panic!("load: {e}"));
    if let Ok(path) = std::env::var("BENCH_STATE") {
        let state = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        core.unserialize(&state)
            .unwrap_or_else(|e| panic!("state: {e}"));
    }
    let fps = core.av_info().fps;
    let mut window = Instant::now();
    let started = Instant::now();
    let pinned = std::env::var_os("BENCH_PIN").is_some();
    // how long each frame took, to find the ones that would be heard as a gap
    let mut took: Vec<f64> = Vec::with_capacity(frames as usize);
    for n in 0..frames {
        // the threads a core starts are started by its first frames, so look again for a while
        if pinned && (n == 0 || n == 60 || n == 600) {
            pin_threads();
        }
        // Start then A, a frame each, every two seconds
        let input = match n % 120 {
            0 => START,
            60 => A,
            _ => 0,
        };
        let began = Instant::now();
        core.run_frame(ButtonMask(input));
        let _ = core.take_audio();
        took.push(began.elapsed().as_secs_f64() * 1000.0);
        if n % 300 == 299 {
            let secs = window.elapsed().as_secs_f64();
            println!(
                "frames {}..{}: {:.1} fps (core asks {fps:.1}), picture {:?}",
                n - 299,
                n,
                300.0 / secs,
                core.video_size()
            );
            window = Instant::now();
        }
    }
    let over = |ms: f64| took.iter().filter(|t| **t > ms).count();
    let worst = took.iter().cloned().fold(0.0, f64::max);
    println!(
        "frames over 20 ms: {}, over 33 ms: {}, over 50 ms: {}, slowest {worst:.1} ms",
        over(20.0),
        over(33.0),
        over(50.0)
    );
    println!(
        "all {frames}: {:.1} fps",
        frames as f64 / started.elapsed().as_secs_f64()
    );
    if let Ok(path) = std::env::var("BENCH_SAVE") {
        let state = core.serialize().unwrap_or_else(|e| panic!("state: {e}"));
        std::fs::write(&path, state).unwrap_or_else(|e| panic!("{path}: {e}"));
        println!("state saved to {path}");
    }
    if let Some(out) = args.get(4) {
        let (w, h) = core.video_size();
        let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
        for px in core.video_xrgb8888().chunks_exact(4) {
            ppm.extend_from_slice(&[px[2], px[1], px[0]]);
        }
        std::fs::write(out, ppm).expect("write the picture");
    }
}
