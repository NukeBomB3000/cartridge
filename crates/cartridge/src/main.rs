#[cfg(not(feature = "host"))]
mod device_app;
#[cfg(feature = "host")]
mod host_app;

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next();
    if first.as_deref() == Some("--hud-faces") {
        let Some(out) = args.next() else {
            eprintln!("cartridge: --hud-faces needs a file");
            std::process::exit(2);
        };
        let root = std::env::var("CARTRIDGE_ROOT").unwrap_or_else(|_| "/mnt/sdcard".into());
        if let Err(e) =
            cartridge::hud_faces::write(std::path::Path::new(&root), std::path::Path::new(&out))
        {
            eprintln!("cartridge: {out}: {e}");
            std::process::exit(1);
        }
        return;
    }
    if first.as_deref() == Some("--init-root") {
        let Some(dir) = args.next() else {
            eprintln!("cartridge: --init-root needs a directory");
            std::process::exit(2);
        };
        cartridge::root::ensure(std::path::Path::new(&dir));
        return;
    }

    #[cfg(feature = "host")]
    host_app::run();

    #[cfg(not(feature = "host"))]
    device_app::run();
}
