use std::path::{Path, PathBuf};

use cartridge_retro::{LibretroCore, MockCore, RetroCore};
use cartridge_store::{Core, GbColors, GbPalette, Platform};

use crate::root;

const CORE_ENV: &str = "CARTRIDGE_CORE";

pub fn dylib_name(core: Core) -> String {
    format!(
        "{}_libretro.{}",
        core.library(),
        std::env::consts::DLL_EXTENSION
    )
}

fn candidates(root: &Path, core: Core) -> Vec<PathBuf> {
    if let Some(named) = std::env::var_os(CORE_ENV) {
        return vec![PathBuf::from(named)];
    }
    let name = dylib_name(core);
    let mut paths = vec![root.join("System").join(&name)];
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        paths.push(dir.join(&name));
        paths.push(dir.join("vendor").join(&name));
    }
    paths.push(Path::new("vendor").join(&name));
    paths
}

pub struct Opened {
    pub core: Box<dyn RetroCore>,
    pub named: bool,
}

/// What a game is opened with besides its core: a handheld's link cable mode, its screen's
/// colour correction and ghosting, and the player it is on a cable. A SNES or PlayStation game
/// has none of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Setup<'a> {
    Handheld {
        serial: &'a str,
        colour: bool,
        ghosting: bool,
        link: Option<u8>,
        /// The named palette a Game Boy only game is drawn in; none, the Game Boy Color's own
        /// colouring of it.
        palette: Option<GbPalette>,
    },
    Console,
}

impl<'a> Setup<'a> {
    /// The serial mode, colour correction, ghosting and cable player the core's options are
    /// set from.
    fn handheld(self) -> (&'a str, bool, bool, Option<u8>) {
        match self {
            Setup::Handheld {
                serial,
                colour,
                ghosting,
                link,
                ..
            } => (serial, colour, ghosting, link),
            Setup::Console => ("auto", false, false, None),
        }
    }

    fn palette(self) -> Option<GbPalette> {
        match self {
            Setup::Handheld { palette, .. } => palette,
            Setup::Console => None,
        }
    }
}

/// Opens `core` for a game, from the content root's System folder or beside cartridge, with the
/// options cartridge sets on it; the mock test pattern when it is nowhere.
pub fn open(root: &Path, core: Core, setup: Setup) -> Opened {
    let paths = candidates(root, core);
    match open_named(root, core, setup, &paths) {
        Some(core) => Opened { core, named: true },
        None => {
            report_missing(core, &paths);
            Opened {
                core: Box::new(stand_in(core)),
                named: false,
            }
        }
    }
}

/// The mock test pattern that stands in for a core that is nowhere, drawing its system's usual
/// picture, so a SNES or PlayStation game without its core is still laid out as one.
fn stand_in(core: Core) -> MockCore {
    match core {
        Core::Mgba | Core::Gpsp => MockCore::new(),
        Core::Snes9x | Core::Supafaust => MockCore::sized(256, 224),
        Core::PcsxRearmed | Core::Swanstation => MockCore::sized(320, 240),
        Core::Dsperate => MockCore::sized(384, 480),
        Core::PceFast => MockCore::sized(256, 240),
        Core::Fceumm => MockCore::sized(256, 224),
        Core::GenesisPlusGx => MockCore::sized(320, 224),
    }
}

/// `open` for a handheld game, with the LCD's ghosting off: each frame as the core drew it.
pub fn open_core(root: &Path, core: Core, serial: &str, colour: bool, link: Option<u8>) -> Opened {
    open(
        root,
        core,
        Setup::Handheld {
            serial,
            colour,
            ghosting: false,
            link,
            palette: None,
        },
    )
}

pub fn open_core_for(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    paths: &[PathBuf],
) -> Box<dyn RetroCore> {
    let setup = Setup::Handheld {
        serial,
        colour,
        ghosting: false,
        link: None,
        palette: None,
    };
    open_named(root, core, setup, paths).unwrap_or_else(|| {
        report_missing(core, paths);
        Box::new(stand_in(core))
    })
}

fn open_named(
    root: &Path,
    core: Core,
    setup: Setup,
    paths: &[PathBuf],
) -> Option<Box<dyn RetroCore>> {
    let bios = root::bios_dir(root);
    let saves = root::saves_dir(root);
    let (serial, colour, ghosting, link) = setup.handheld();
    for path in paths {
        if !path.exists() {
            continue;
        }
        // the options are there before the core starts, for one that reads them as it does
        let mut options = core_options(core, serial, root::has_real_bios(root), colour, ghosting);
        options.extend(palette_options(core, setup.palette()));
        let early: Vec<(&str, &str)> = options.iter().map(|(k, v)| (*k, v.as_str())).collect();
        match LibretroCore::open_with_options(path, &bios, &saves, &early) {
            Ok(mut opened) => {
                apply_core_options(
                    &mut opened,
                    core,
                    serial,
                    root::has_real_bios(root),
                    colour,
                    ghosting,
                );
                for (key, value) in palette_options(core, setup.palette()) {
                    opened.set_option(key, &value);
                }
                // the DualShock, as only it rumbles; its sticks stay centred
                if core == Core::PcsxRearmed {
                    opened.set_port_device(cartridge_retro::DEVICE_DUALSHOCK);
                }
                if let Some(player) = link {
                    apply_link_options(&mut opened, core, player);
                }
                eprintln!("cartridge: core {}", path.display());
                return Some(Box::new(opened));
            }
            Err(e) => eprintln!("cartridge: {}: {e}", path.display()),
        }
    }
    None
}

fn report_missing(core: Core, paths: &[PathBuf]) {
    eprintln!(
        "cartridge: no {} core found, running the mock test pattern instead. Looked in: {}",
        core.as_str(),
        paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

pub fn colour_option(which: Core, on: bool) -> Option<(&'static str, &'static str)> {
    match which {
        Core::Mgba => Some(("mgba_color_correction", if on { "Auto" } else { "OFF" })),
        Core::Gpsp => Some((
            "gpsp_color_correction",
            if on { "enabled" } else { "disabled" },
        )),
        Core::Snes9x
        | Core::Supafaust
        | Core::PcsxRearmed
        | Core::Swanstation
        | Core::Dsperate
        | Core::PceFast
        | Core::Fceumm
        | Core::GenesisPlusGx => None,
    }
}

/// The option that blends each frame with the last, as the handhelds' LCDs ghosted, spelt the
/// way each of their cores declares it: mGBA fades each frame into the next as an LCD's response
/// did, gpSP its frame mixing, Genesis Plus GX its Game Gear's LCD filter.
pub fn ghosting_option(which: Core, on: bool) -> Option<(&'static str, &'static str)> {
    match which {
        Core::Mgba => Some((
            "mgba_interframe_blending",
            if on { "lcd_ghosting" } else { "OFF" },
        )),
        Core::Gpsp => Some(("gpsp_frame_mixing", if on { "enabled" } else { "disabled" })),
        // its filter is for the Game Gear's LCD, and only ever on for a Game Gear game
        Core::GenesisPlusGx => Some((
            "genesis_plus_gx_lcd_filter",
            if on { "enabled" } else { "disabled" },
        )),
        Core::Snes9x
        | Core::Supafaust
        | Core::PcsxRearmed
        | Core::Swanstation
        | Core::Dsperate
        | Core::PceFast
        | Core::Fceumm => None,
    }
}

/// The options that draw a Game Boy only game in `palette` on mGBA: as the original Game Boy,
/// in that palette. Set after `core_options`, whose Game Boy Color colouring they replace.
pub fn palette_options(which: Core, palette: Option<GbPalette>) -> Vec<(&'static str, String)> {
    match (which, palette) {
        (Core::Mgba, Some(p)) => vec![
            ("mgba_gb_model", "Game Boy".to_string()),
            ("mgba_gb_colors_preset", "0".to_string()),
            ("mgba_gb_colors", p.core_name().to_string()),
        ],
        _ => Vec::new(),
    }
}

/// The palette a game is opened in: the one chosen, for a Game Boy only game played alone; none,
/// its Game Boy Color colouring, for any other or for Auto.
pub fn palette_for(
    chosen: GbColors,
    platform: Platform,
    rom: &Path,
    link: Option<u8>,
) -> Option<GbPalette> {
    let GbColors::Palette(p) = chosen else {
        return None;
    };
    (link.is_none()
        && platform == Platform::Gb
        && cartridge_store::gb::class(rom) == cartridge_store::gb::Class::Original)
        .then_some(p)
}

pub fn apply_link_options(core: &mut LibretroCore, which: Core, player: u8) {
    if which != Core::Mgba {
        return;
    }
    core.set_option("mgba_link", "on");
    core.set_option("mgba_link_player", &player.to_string());
    eprintln!("cartridge: core: link mode on, player {player}");
}

pub fn apply_core_options(
    core: &mut LibretroCore,
    which: Core,
    serial: &str,
    bios: bool,
    colour: bool,
    ghosting: bool,
) {
    for (key, value) in core_options(which, serial, bios, colour, ghosting) {
        core.set_option(key, &value);
    }
}

/// Every option cartridge sets on `which`, in the order it sets them, each spelt as that core
/// declares it. `serial`, the BIOS, `colour` and `ghosting` only reach the handhelds' cores.
///
/// Frame skipping is asked for wherever the core has it, so fast-forward can skip frames by
/// reporting an underrun: mGBA and gpSP name it `_frameskip`, pcsx_rearmed
/// `_frameskip_type`; Snes9x has none and skips a frame the frontend says not to draw, and
/// Supafaust skips none. The SNES picture is kept to its 224 lines rather than the 239 a few
/// games show, so it is exactly twice that on every canvas. Supafaust is left to run its
/// threads wherever the system puts them: by itself it would pin the emulator thread to the
/// second CPU and its PPU thread to the first, where the render thread and the kernel's
/// interrupts run. The PlayStation runs its own BIOS from BIOS/ when one is there and its
/// built-in one otherwise, and keeps memory card 1 as the game's save, in Saves/PS1; the
/// second card, which pcsx_rearmed would keep in a file of its own in Saves, is left out.
/// Beetle PCE Fast skips frames as the handhelds' cores do, through `pce_fast_frameskip`. FCEUmm
/// has no frame skipping of its own and skips a frame the frontend says not to draw, as Snes9x
/// does; its picture is kept to the 224 lines a television showed, 8 cut from the top and the
/// bottom, so it is exactly twice that on every canvas as the SNES's is. Genesis Plus GX skips
/// frames through `genesis_plus_gx_frameskip`, and picks each game's pad itself, three buttons
/// for a game a six button pad confuses.
pub fn core_options(
    which: Core,
    serial: &str,
    bios: bool,
    colour: bool,
    ghosting: bool,
) -> Vec<(&'static str, String)> {
    let mut set: Vec<(&'static str, String)> = Vec::new();
    let mut put = |key: &'static str, value: &str| set.push((key, value.to_string()));
    match which {
        Core::Mgba => {
            put("mgba_frameskip", "auto");
            put("mgba_sgb_borders", "OFF");
            // answers the Game Boy Player logo, so Pokemon Pinball R/S rumbles
            put("mgba_force_gbp", "ON");
            put("mgba_gb_colors_preset", "1");
            put("mgba_gb_colors", "GBC Dark Green →A");
        }
        Core::Gpsp => {
            put("gpsp_frameskip", "auto");
            put("gpsp_serial", serial);
            if bios {
                put("gpsp_boot_mode", "bios");
            }
        }
        Core::Snes9x => put("snes9x_overscan", "enabled"),
        Core::Supafaust => {
            put("supafaust_thread_affinity_emu", "0x0");
            put("supafaust_thread_affinity_ppu", "0x0");
        }
        Core::PcsxRearmed => {
            put("pcsx_rearmed_frameskip_type", "auto");
            put("pcsx_rearmed_bios", "auto");
            put("pcsx_rearmed_memcard1", "libretro");
            put("pcsx_rearmed_memcard2", "none");
            put("pcsx_rearmed_vibration", "enabled");
        }
        // SwanStation draws with the GPU through the GLES 3 context it asks the frontend for,
        // at the PlayStation's own resolution, falling back on software where there is none,
        // and runs the BIOS it finds among BIOS/ by its hash; it has none built in
        Core::Swanstation => {
            put("swanstation_GPU_Renderer", "Auto");
            put("swanstation_GPU_ResolutionScale", "1");
            put("swanstation_CPU_ExecutionMode", "Recompiler");
            put("swanstation_Main_ApplyGameSettings", "true");
        }
        // DSperate lays the top screen out large over the bottom one, its JIT on; its stylus
        // moves while L2 is held with the d-pad and taps with R2, the screens swap with L3
        Core::Dsperate => {
            put("dsperate_layout", "Top Dominant");
            put("dsperate_cpu_engine", "JIT (AArch64)");
            put("dsperate_stylus_control", "Hold L2 + D-Pad");
            put("dsperate_touch_button", "R2");
            put("dsperate_swap_button", "L3");
            put("dsperate_mic_button", "Off");
            put("dsperate_show_cursor", "On");
            // skips drawing a frame while the sound runs short, as cartridge asks
            put("dsperate_frameskip", "Auto");
        }
        Core::PceFast => put("pce_fast_frameskip", "auto"),
        Core::Fceumm => {
            put("fceumm_overscan_v_top", "8");
            put("fceumm_overscan_v_bottom", "8");
        }
        Core::GenesisPlusGx => put("genesis_plus_gx_frameskip", "auto"),
    }
    if let Some((key, value)) = colour_option(which, colour) {
        put(key, value);
    }
    if let Some((key, value)) = ghosting_option(which, ghosting) {
        put(key, value);
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn each_core_spells_colour_correction_its_own_way() {
        assert_eq!(
            colour_option(Core::Mgba, true),
            Some(("mgba_color_correction", "Auto"))
        );
        assert_eq!(
            colour_option(Core::Mgba, false),
            Some(("mgba_color_correction", "OFF"))
        );
        assert_eq!(
            colour_option(Core::Gpsp, true),
            Some(("gpsp_color_correction", "enabled"))
        );
        assert_eq!(
            colour_option(Core::Gpsp, false),
            Some(("gpsp_color_correction", "disabled"))
        );
    }

    #[test]
    fn each_handheld_core_spells_lcd_ghosting_its_own_way() {
        assert_eq!(
            ghosting_option(Core::Mgba, true),
            Some(("mgba_interframe_blending", "lcd_ghosting"))
        );
        assert_eq!(
            ghosting_option(Core::Mgba, false),
            Some(("mgba_interframe_blending", "OFF"))
        );
        assert_eq!(
            ghosting_option(Core::Gpsp, true),
            Some(("gpsp_frame_mixing", "enabled"))
        );
        assert_eq!(
            ghosting_option(Core::Gpsp, false),
            Some(("gpsp_frame_mixing", "disabled"))
        );
        assert_eq!(
            ghosting_option(Core::GenesisPlusGx, true),
            Some(("genesis_plus_gx_lcd_filter", "enabled"))
        );
        for which in [Core::Snes9x, Core::Supafaust, Core::PcsxRearmed] {
            assert_eq!(ghosting_option(which, true), None, "{which:?}");
        }
    }

    #[test]
    fn the_handheld_cores_are_set_to_blend_frames_only_while_ghosting_is_on() {
        for (which, key, on, off) in [
            (
                Core::Mgba,
                "mgba_interframe_blending",
                "lcd_ghosting",
                "OFF",
            ),
            (Core::Gpsp, "gpsp_frame_mixing", "enabled", "disabled"),
        ] {
            let value = |ghosting| {
                core_options(which, "auto", false, false, ghosting)
                    .into_iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, v)| v)
            };
            assert_eq!(value(true).as_deref(), Some(on), "{which:?}");
            assert_eq!(value(false).as_deref(), Some(off), "{which:?}");
        }
        for which in [Core::Snes9x, Core::Supafaust, Core::PcsxRearmed] {
            assert!(
                !core_options(which, "auto", false, false, true)
                    .iter()
                    .any(|(k, _)| k.contains("blending") || k.contains("frame_mixing")),
                "{which:?} was told to blend frames"
            );
        }
    }

    fn keys(which: Core) -> Vec<&'static str> {
        core_options(which, "auto", true, true, true)
            .into_iter()
            .map(|(k, _)| k)
            .collect()
    }

    #[test]
    fn every_option_is_spelt_with_its_own_core_s_prefix() {
        for which in Core::ALL {
            let prefix = format!("{}_", which.as_str());
            for key in keys(which) {
                assert!(key.starts_with(&prefix), "{key} set on {which:?}");
            }
        }
    }

    #[test]
    fn frame_skipping_is_asked_for_by_each_core_s_own_name_for_it() {
        let skip = |which| {
            core_options(which, "auto", false, false, false)
                .into_iter()
                .find(|(k, _)| k.contains("frameskip"))
        };
        assert_eq!(skip(Core::Mgba), Some(("mgba_frameskip", "auto".into())));
        assert_eq!(skip(Core::Gpsp), Some(("gpsp_frameskip", "auto".into())));
        assert_eq!(
            skip(Core::PcsxRearmed),
            Some(("pcsx_rearmed_frameskip_type", "auto".into())),
            "pcsx_rearmed's _frameskip is an interval, not a mode"
        );
        assert_eq!(skip(Core::Snes9x), None, "Snes9x has no frame skipping");
        assert_eq!(
            skip(Core::Supafaust),
            None,
            "Supafaust has no frame skipping"
        );
    }

    #[test]
    fn the_consoles_take_no_handheld_options() {
        for which in [Core::Snes9x, Core::Supafaust, Core::PcsxRearmed] {
            assert_eq!(
                core_options(which, "rfu", true, true, true),
                core_options(which, "auto", false, false, false),
                "{which:?} took a serial mode, a BIOS boot, colour correction or ghosting"
            );
        }
        assert!(core_options(Core::PcsxRearmed, "auto", false, false, false)
            .contains(&("pcsx_rearmed_memcard1", "libretro".into())));
    }

    #[test]
    fn a_missing_console_core_is_stood_in_for_at_its_system_s_size() {
        assert_eq!(stand_in(Core::Mgba).video_size(), (240, 160));
        assert_eq!(stand_in(Core::Snes9x).video_size(), (256, 224));
        assert_eq!(stand_in(Core::Supafaust).video_size(), (256, 224));
        assert_eq!(stand_in(Core::PcsxRearmed).video_size(), (320, 240));
    }

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn candidates_search_the_named_cores_own_filename_only() {
        let _g = lock();
        std::env::remove_var(CORE_ENV);

        let root = Path::new("/root");
        let mgba = candidates(root, Core::Mgba);
        let gpsp = candidates(root, Core::Gpsp);
        assert_ne!(mgba, gpsp, "the two cores searched the same paths");
        assert!(!mgba.is_empty());
        assert!(!gpsp.is_empty());
        for path in &mgba {
            let name = path.file_name().unwrap().to_str().unwrap();
            assert!(name.starts_with("mgba_libretro"), "{name} is not mGBA's");
        }
        for path in &gpsp {
            let name = path.file_name().unwrap().to_str().unwrap();
            assert!(name.starts_with("gpsp_libretro"), "{name} is not gpSP's");
        }
    }

    #[test]
    fn candidates_search_the_roots_own_system_directory() {
        let _g = lock();
        std::env::remove_var(CORE_ENV);
        let root = Path::new("/some/content/root");
        assert_eq!(
            candidates(root, Core::Gpsp)[0],
            root.join("System").join(dylib_name(Core::Gpsp)),
        );
    }

    #[test]
    fn the_env_override_ignores_which_core_was_asked_for() {
        let _g = lock();
        std::env::set_var(CORE_ENV, "/dev/null/named-core");
        let root = Path::new("/root");
        let mgba = candidates(root, Core::Mgba);
        let gpsp = candidates(root, Core::Gpsp);
        std::env::remove_var(CORE_ENV);
        assert_eq!(mgba, vec![PathBuf::from("/dev/null/named-core")]);
        assert_eq!(
            mgba, gpsp,
            "the override stopped winning for one of the cores"
        );
    }
}
