//! Runs a libretro core on a game with no picture or sound, pressing Start and A now and then
//! to get past title screens, and says how fast it ran: for judging a core on the device before
//! it has a shelf.
//!
//!   bench CORE ROM [FRAMES] [OUT.ppm]
//!
//! BENCH_OPTIONS="key=value,key=value" sets core options before the game loads, and BENCH_STATE
//! names a save state to put in once it has, to time a scene rather than a title screen.

use std::path::Path;
use std::time::Instant;

use cartridge_retro::{ButtonMask, LibretroCore, RetroCore};

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
    for n in 0..frames {
        // Start then A, a frame each, every two seconds
        let input = match n % 120 {
            0 => START,
            60 => A,
            _ => 0,
        };
        core.run_frame(ButtonMask(input));
        let _ = core.take_audio();
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
    println!(
        "all {frames}: {:.1} fps",
        frames as f64 / started.elapsed().as_secs_f64()
    );
    if let Some(out) = args.get(4) {
        let (w, h) = core.video_size();
        let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
        for px in core.video_xrgb8888().chunks_exact(4) {
            ppm.extend_from_slice(&[px[2], px[1], px[0]]);
        }
        std::fs::write(out, ppm).expect("write the picture");
    }
}
