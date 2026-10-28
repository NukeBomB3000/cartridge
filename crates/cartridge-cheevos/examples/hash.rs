//! Hashes a game the way the frontend does when it loads one, for finding why it takes the
//! process down:
//!
//!   hash CONSOLE FILE [thread]
//!
//! CONSOLE is RetroAchievements' number (12 for a PlayStation); `thread` runs it on a spawned
//! thread with the default stack, as the frontend does, rather than the main one.

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: hash CONSOLE FILE [thread]");
        std::process::exit(2);
    }
    let console: u32 = args[1].parse().expect("a console number");
    let path = PathBuf::from(&args[2]);
    let run = move || {
        use std::io::Read;
        let mut header = vec![0u8; 512];
        let read = std::fs::File::open(&path).and_then(|mut f| f.read(&mut header));
        if let Ok(read) = read {
            header.truncate(read);
            println!(
                "console_for: {}",
                cartridge_cheevos::game::console_for(&path, &header)
            );
        }
        println!(
            "hash: {:?}",
            cartridge_cheevos::game::hash_file(console, &path)
        );
    };
    if args.get(3).is_some() {
        std::thread::Builder::new()
            .name("hash".into())
            .spawn(run)
            .unwrap()
            .join()
            .unwrap();
    } else {
        run();
    }
}
