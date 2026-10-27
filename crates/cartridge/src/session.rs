use std::path::PathBuf;
use std::time::Duration;

use cartridge_input::{Action, Gestures, Millis, RawEvent};
use cartridge_retro::Rumble;
use cartridge_store::Platform;
use cartridge_ui::FfState;

use crate::app::{App, Phase};
use crate::audio::{open_sink, AudioSink, Ring, Sfx, OUTPUT_HZ};
use crate::core::{open, Setup};
use crate::emu::{CoreState, EmuHandle, Speed};
use crate::frames::FrameRef;
use crate::input::Pad;
use crate::persist;
use crate::prefetch::Prefetch;

/// How many frames a core runs before the card stops noting it as opening: two seconds, well
/// past the first frame, where a core that cannot take its options gives up.
const SETTLED_FRAMES: u64 = 120;

/// The shortest a buzz runs, and the longest, in milliseconds.
pub const RUMBLE_MIN_MS: Millis = 100;
pub const RUMBLE_MAX_MS: Millis = 150;

pub struct Session {
    root: PathBuf,
    app: App,
    emu: Option<EmuHandle>,
    sink: Box<dyn AudioSink>,
    gestures: Gestures,
    pad: Pad,
    rewinding: bool,
    fast: bool,
    motor: u16,
    /// When the buzz running now began, and how strong the game asks for it.
    pulse: Option<(Millis, u16)>,
    /// Until when the launcher's buzz runs.
    ui_buzz: Millis,
    reloading: bool,
    driven: bool,
    /// The system and stem of the game the core was last opened for.
    running: Option<(Platform, String)>,
    /// The card notes the core as opening, until it has run a moment.
    starting: bool,
    /// The cart the shelf rests on, read ahead.
    prefetch: Prefetch,
    /// The equalizer's test sound while it plays, and how far into it the ring has been fed.
    test_sound: Option<(Vec<i16>, usize)>,
}

impl Drop for Session {
    /// A session put away, as at power-off, leaves no game noted as opening: only one that went
    /// down with its core, and so never got here, does.
    fn drop(&mut self) {
        if self.starting {
            persist::clear_starting(&self.root);
        }
        // the CPU and the GPU left free for whatever runs after
        self.app.set_cpu_cap(None);
        self.app.set_gpu_boost(false);
    }
}

impl Session {
    pub fn boot(root: PathBuf) -> Self {
        let mut sink: Box<dyn AudioSink> = open_sink();
        if let Err(e) = sink.open(OUTPUT_HZ) {
            eprintln!("cartridge: audio: {e}");
        }
        Session {
            app: App::boot(&root),
            root,
            emu: None,
            sink,
            gestures: Gestures::new(),
            pad: Pad::default(),
            rewinding: false,
            fast: false,
            motor: 0,
            pulse: None,
            ui_buzz: 0,
            reloading: false,
            driven: false,
            running: None,
            starting: false,
            prefetch: Prefetch::default(),
            test_sound: None,
        }
    }

    pub fn play_sfx(&mut self, sfx: Sfx) {
        let ring = self.sink.ring();
        let rate = ring.sample_rate();
        if rate == 0 {
            return;
        }
        let mut samples = sfx.render(rate);
        crate::audio::volume::apply(&mut samples, self.app.output_volume());
        ring.mix(&samples);
    }

    /// The ring's equalizer at the bands for what plays out now, and the test sound fed to it
    /// while the EQUALIZER screen plays it: kept a few frames ahead, as a core keeps its sound,
    /// so the screen's own blips mix over it rather than cut it off.
    fn sync_equalizer(&mut self) {
        let ring = self.sink.ring();
        let gains = self.app.eq_gains();
        if ring.eq() != gains {
            ring.set_eq(gains);
        }
        if !self.app.eq_test_playing() || ring.sample_rate() == 0 {
            self.test_sound = None;
            return;
        }
        let (clip, at) = self
            .test_sound
            .get_or_insert_with(|| (crate::audio::test_sound::test_sound(&self.root), 0));
        if clip.is_empty() {
            return;
        }
        let ahead = ring.capacity_frames() * 3 / 4;
        let volume = self.app.output_volume();
        while ring.queued_frames() < ahead {
            let n = (ahead - ring.queued_frames()).min(1024) * 2;
            let mut chunk = Vec::with_capacity(n);
            while chunk.len() < n {
                let take = (n - chunk.len()).min(clip.len() - *at);
                chunk.extend_from_slice(&clip[*at..*at + take]);
                *at = (*at + take) % clip.len();
            }
            crate::audio::volume::apply(&mut chunk, volume);
            ring.push(&chunk);
        }
    }

    pub fn audio_queued(&self) -> usize {
        self.sink.ring().queued_frames()
    }

    pub fn audio_ring(&self) -> std::sync::Arc<Ring> {
        self.sink.ring()
    }

    pub fn rumble(&mut self, strength: u16) {
        if strength == self.motor {
            return;
        }
        self.motor = strength;
        self.app.set_rumble(strength);
    }

    pub fn core_rumble(&self) -> Option<&Rumble> {
        self.emu.as_ref().map(EmuHandle::rumble)
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    pub fn emu(&self) -> Option<&EmuHandle> {
        self.emu.as_ref()
    }

    pub fn set_driven(&mut self, driven: bool) {
        self.driven = driven;
        if let Some(emu) = &self.emu {
            emu.set_driven(driven);
        }
    }

    /// What the frontend's own clock ticks at: the game's period while its core is paced by
    /// the frontend, the UI's otherwise.
    pub fn frame_clock(&self) -> crate::pacing::FrameClock {
        self.emu
            .as_ref()
            .filter(|emu| emu.locked())
            .map_or_else(crate::pacing::FrameClock::default, EmuHandle::frame_clock)
    }

    pub fn step_emulator(&self, present: Duration, timeout: Duration) -> bool {
        self.emu.as_ref().is_some_and(|emu| {
            emu.tick(present);
            emu.wait_frame(timeout)
        })
    }

    pub fn frame(&self) -> Option<FrameRef> {
        self.emu.as_ref().and_then(|e| e.latest_frame())
    }

    pub fn core_settling(&self) -> bool {
        self.emu
            .as_ref()
            .is_some_and(|e| e.state() == CoreState::Loading)
    }

    pub fn has_core(&self) -> bool {
        self.emu.is_some()
    }

    pub fn frame_ready(&self) -> bool {
        self.emu.as_ref().is_some_and(EmuHandle::frame_ready)
    }

    pub fn frames_published(&self) -> u64 {
        self.emu.as_ref().map_or(0, EmuHandle::published_count)
    }

    pub fn observed_speed(&self) -> Option<Speed> {
        self.emu.as_ref().map(EmuHandle::observed_speed)
    }

    pub fn frames_taken(&self) -> u64 {
        self.emu.as_ref().map_or(0, EmuHandle::frames_taken)
    }

    pub fn game_visible(&self) -> bool {
        self.app.game_visible()
    }

    pub fn feed(&mut self, events: impl IntoIterator<Item = RawEvent>, now: Millis) {
        let keys = self.app.keys();
        self.pad.set_keys(keys);
        let mut actions = self.gestures.set_keys(keys);
        for ev in events {
            actions.extend(self.gestures.feed(ev, now));
        }
        actions.extend(self.gestures.tick(now));
        for action in actions {
            self.act(action);
        }
        self.sync_pad();
    }

    fn sync_pad(&mut self) {
        for btn in self.app.taken_buttons() {
            self.pad.apply(Action::GbaUp(*btn));
        }
        if let Some(emu) = &self.emu {
            emu.set_input(self.pad.mask());
        }
    }

    fn bridge_link(&mut self, f: impl FnOnce(&mut App)) {
        let had_link = self.app.link_active();
        f(&mut self.app);
        if had_link && !self.app.link_active() {
            if let Some(emu) = &self.emu {
                emu.end_link();
            }
        }
    }

    fn act(&mut self, action: Action) {
        if trace() {
            eprintln!("cartridge: {action:?} in {:?}", self.app.phase());
        }
        match action {
            // over the guide L2 and R2 size its text instead
            Action::RewindStart if !self.app.guide_open() => self.rewinding = true,
            Action::RewindStop => self.rewinding = false,
            Action::FfStart if !self.app.guide_open() => self.fast = true,
            Action::FfStop => self.fast = false,
            _ => {}
        }
        let menu = self.overlaid();
        self.bridge_link(|app| app.apply(action));
        if matches!(
            action,
            Action::VolumeUp | Action::VolumeDown | Action::MuteToggle
        ) {
            if let Some(emu) = &self.emu {
                emu.set_volume(self.app.output_volume());
            }
        }
        if menu || self.overlaid() {
            self.pad.clear();
        } else if self.app.takes_from_the_game(action) {
            if let Action::GbaDown(btn) | Action::GbaUp(btn) = action {
                self.pad.apply(Action::GbaUp(btn));
            }
        } else {
            self.pad.apply(action);
        }
        self.sync_rumble();
    }

    pub fn update(&mut self, dt: f32) {
        self.bridge_link(|app| app.update(dt));
        if let Some(emu) = &self.emu {
            emu.set_volume(self.app.output_volume());
            emu.set_blend(self.app.frame_blend());
        }
        // a game is held to its system's clock; fast forward, rewind and the menus get it all
        let playing = matches!(self.app.phase(), Phase::Playing { .. }) && self.emu.is_some();
        let cap = (playing && !self.fast && !self.rewinding)
            .then(|| self.app.platform().cpu_khz())
            .flatten();
        self.app.set_cpu_cap(cap);
        // SwanStation draws with the GPU and waits on it each frame
        self.app
            .set_gpu_boost(playing && self.app.core() == cartridge_store::Core::Swanstation);
        if let Some((client_id, transport)) = self.app.take_link_transport() {
            match &self.emu {
                Some(emu) => match self.app.link_player() {
                    Some(player) => emu.begin_cable(player, transport),
                    None => emu.begin_link(client_id, transport),
                },
                None => eprintln!("cartridge: link: a transport arrived with no core to run it"),
            }
        }
        if let Some(on) = self.app.take_colour_correction() {
            if let Some((key, value)) = crate::core::colour_option(self.app.core(), on) {
                if let Some(emu) = &self.emu {
                    emu.set_option(key, value);
                }
            }
        }
        if let Some(on) = self.app.take_lcd_ghosting() {
            if let Some((key, value)) = crate::core::ghosting_option(self.app.core(), on) {
                if let Some(emu) = &self.emu {
                    emu.set_option(key, value);
                }
            }
        }
        if let Some(palette) = self.app.take_palette() {
            if let Some(emu) = &self.emu {
                emu.set_option("mgba_gb_colors", palette.core_name());
            }
        }
        if let Some(disc) = self.app.take_disc_swap() {
            if let Some(emu) = &self.emu {
                emu.set_disc(disc);
            }
        }
        if let Some((stem, serial)) = self.app.take_link_reload() {
            self.reload_for_link(&stem, serial);
        }
        if self.app.link_active() {
            if self.emu.as_ref().is_some_and(EmuHandle::bios_mismatch) {
                self.bridge_link(|app| app.bios_mismatch());
            } else if self.emu.as_ref().is_some_and(EmuHandle::peer_ended) {
                self.bridge_link(|app| app.peer_ended());
            } else if self.emu.as_ref().is_some_and(EmuHandle::link_lost) {
                self.app.peer_lost();
            }
        }
        self.sync_equalizer();
        if let Some(sfx) = self.app.take_sfx() {
            self.play_sfx(sfx);
        }
        if let Some(ms) = self.app.take_buzz() {
            self.ui_buzz = self.app.now() + ms;
        }
        self.sync_core();
        self.settle_start();
        self.sync_reload();
        self.keep_swapped_disc();
        if !self.has_core() {
            for action in self.gestures.drop_ff_latch() {
                self.act(action);
            }
        }
        self.app
            .set_game_ready(self.emu.as_ref().is_some_and(EmuHandle::has_published));
        self.sync_speed();
        self.sync_rewind_hud();
        self.sync_ff_hud();
        self.sync_rumble();
        self.sync_pad();
        let now = self.app.now();
        self.prefetch.browse(self.app.browsed_rom(), now);
    }

    /// The ROM being read ahead, if one is.
    pub fn prefetching(&self) -> Option<&std::path::Path> {
        self.prefetch.reading()
    }

    /// Runs the motor as the game asks, each buzz for at least `RUMBLE_MIN_MS`, so a game's
    /// frame-long taps are felt, and at most `RUMBLE_MAX_MS`, so a GameCube-length pulse does
    /// not drone on.
    fn sync_rumble(&mut self) {
        let asked = match &self.emu {
            Some(emu) if self.playing() && self.app.rumble_enabled() => {
                match emu.rumble().strength() {
                    0 => 0,
                    _ => self.app.rumble_strength(),
                }
            }
            _ => {
                self.pulse = None;
                let buzz = self.app.rumble_enabled() && self.app.now() < self.ui_buzz;
                let want = if buzz { self.app.rumble_strength() } else { 0 };
                return self.rumble(want);
            }
        };
        let now = self.app.now();
        let want = match (asked, self.pulse) {
            (0, Some((at, strength))) if now.saturating_sub(at) < RUMBLE_MIN_MS => strength,
            (0, _) => {
                self.pulse = None;
                0
            }
            (asked, Some((at, _))) => {
                self.pulse = Some((at, asked));
                if now.saturating_sub(at) < RUMBLE_MAX_MS {
                    asked
                } else {
                    0
                }
            }
            (asked, None) => {
                self.pulse = Some((now, asked));
                asked
            }
        };
        self.rumble(want);
    }

    fn sync_ff_hud(&mut self) {
        let ff = match (self.actually_fast_forwarding(), self.gestures.ff_latched()) {
            (false, _) => FfState::Off,
            (true, false) => FfState::Held,
            (true, true) => FfState::Latched,
        };
        self.app.set_ff(ff);
    }

    fn sync_rewind_hud(&mut self) {
        let fill = self
            .actually_rewinding()
            .then(|| self.emu.as_ref().map(EmuHandle::rewind_fill))
            .flatten();
        match fill {
            Some(fill) => self.app.show_rewind(fill),
            None => self.app.hide_rewind(),
        }
    }

    fn actually_rewinding(&self) -> bool {
        self.rewinding && self.playing() && self.app.may_rewind()
    }

    fn actually_fast_forwarding(&self) -> bool {
        self.fast && self.playing() && self.app.may_fast_forward()
    }

    fn inserting(&self) -> bool {
        matches!(self.app.phase(), Phase::Inserting { .. })
    }

    fn showing_polaroids(&self) -> bool {
        matches!(self.app.phase(), Phase::Polaroids { .. })
    }

    fn overlaid(&self) -> bool {
        self.showing_polaroids() || self.held()
    }

    fn playing(&self) -> bool {
        matches!(self.app.phase(), Phase::Playing { .. }) && !self.held()
    }

    fn held(&self) -> bool {
        self.app.power_shown() || self.app.game_menu_open() || self.app.shutting_down()
    }

    fn dozing(&self) -> bool {
        matches!(self.app.phase(), Phase::Doze { .. })
    }

    fn ejecting(&self) -> bool {
        matches!(self.app.phase(), Phase::Ejecting { .. })
    }

    fn sync_speed(&self) {
        if let Some(emu) = &self.emu {
            emu.set_fast_steps(u32::from(self.app.ff_speed()));
            emu.set_ff_sound(self.app.ff_sound());
            emu.set_speed(
                if self.inserting()
                    || self.ejecting()
                    || self.showing_polaroids()
                    || self.dozing()
                    || (self.held() && !self.app.link_active())
                {
                    Speed::Paused
                } else if self.actually_fast_forwarding() {
                    Speed::Fast
                } else {
                    Speed::Normal
                },
            );
            emu.set_rewinding(self.actually_rewinding());
        }
    }

    fn no_core() -> bool {
        std::env::var_os("CARTRIDGE_NO_CORE").is_some_and(|v| v != "0")
    }

    fn sync_core(&mut self) {
        if Self::no_core() {
            return;
        }
        let stem = match self.app.phase() {
            Phase::Shelf => {
                self.emu = None;
                return;
            }
            Phase::Inserting { cart, .. } => cart.clone(),
            _ => return,
        };
        if self.emu.is_none() {
            let (_, serial) = self.app.link_mode(&stem);
            self.spawn_core(&stem, serial);
        }
        match self.emu.as_ref().map(EmuHandle::state) {
            Some(CoreState::Loading) => {}
            Some(CoreState::Ready) => self.app.on_core_ready(),
            Some(CoreState::Failed) | None => {
                self.emu = None;
                self.app.on_core_failed();
            }
        }
    }

    fn spawn_core(&mut self, stem: &str, serial: &'static str) {
        let Some((rom, platform)) = self
            .app
            .seated_cart()
            .filter(|c| c.stem == stem)
            .map(|c| (c.rom.clone(), c.platform))
        else {
            return;
        };
        let core = cartridge_store::core_for_platform(&self.root, stem, platform);
        self.app.set_core(core);
        self.app.set_platform(platform);
        self.app
            .set_video_mode(crate::video_mode::video_mode_for(&self.root, platform));
        self.app
            .set_look(crate::shader::shader_for(&self.root, platform));
        self.app.set_link_loaded(serial);
        let resume = (!self.app.starting_clean())
            .then(|| persist::read_resume(&self.root, platform, core, stem))
            .flatten();
        let player = self.app.link_player();
        let palette = crate::core::palette_for(self.app.gb_palette(), platform, &rom, player);
        self.app.set_palette_live(palette.is_some());
        let setup = match platform {
            Platform::Snes
            | Platform::Ps1
            | Platform::Pce
            | Platform::Nes
            | Platform::Md
            | Platform::Sms
            | Platform::Nds => Setup::Console,
            // the Game Gear has no cable or palette here, only its LCD's ghosting
            Platform::Gg => Setup::Handheld {
                serial: "auto",
                colour: false,
                ghosting: self.app.lcd_ghosting(),
                link: None,
                palette: None,
            },
            Platform::Gba | Platform::Gb | Platform::Gbc => Setup::Handheld {
                serial,
                colour: self.app.colour_correction(),
                ghosting: self.app.lcd_ghosting(),
                link: player,
                palette,
            },
        };
        let opened = open(&self.root, core, setup);
        self.app.set_named_core(opened.named);
        let mut game = opened.core;
        // a game put away on its second disc comes back on it, for its resume state is of that
        // disc; a game started afresh starts from the first
        if let Some((index, path)) = resume
            .is_some()
            .then(|| persist::read_disc(&self.root, platform, stem))
            .flatten()
        {
            game.set_initial_disc(index, &path);
            self.app.set_disc_in(stem, index);
        }
        let sav = persist::read_sav(&self.root, platform, stem);
        persist::mark_starting(&self.root, platform, stem);
        self.starting = true;
        let ring = self.sink.ring();
        let emu = EmuHandle::spawn_for(platform, game, rom, ring, sav, resume, player);
        emu.set_volume(self.app.output_volume());
        emu.set_driven(self.driven);
        self.app.set_snapshot(Box::new(emu.snapshot()));
        self.emu = Some(emu);
        self.running = Some((platform, stem.to_string()));
    }

    /// Takes the note of the core opening off the card once it has run a moment, or once there
    /// is no core: it opened, or failed to, without taking cartridge down.
    fn settle_start(&mut self) {
        let settled = self
            .emu
            .as_ref()
            .is_none_or(|emu| emu.published_count() >= SETTLED_FRAMES);
        if self.starting && settled {
            persist::clear_starting(&self.root);
            self.starting = false;
        }
    }

    /// Keeps the disc a swap put in, so that the game comes back on it.
    fn keep_swapped_disc(&mut self) {
        let (Some(emu), Some((platform, stem))) = (&self.emu, &self.running) else {
            return;
        };
        if let Some((index, path)) = emu.take_swapped_disc() {
            if let Err(e) = persist::write_disc(&self.root, *platform, stem, index, &path) {
                eprintln!("cartridge: disc: could not keep disc {index} of {stem}: {e}");
            }
        }
    }

    fn reload_for_link(&mut self, stem: &str, serial: &'static str) {
        eprintln!("cartridge: link: loading {stem} again with gpsp_serial={serial}");
        if self.emu.is_some() {
            self.app.flush_resume();
        }
        self.emu = None;
        self.spawn_core(stem, serial);
        self.reloading = true;
    }

    fn sync_reload(&mut self) {
        if !self.reloading {
            return;
        }
        match self.emu.as_ref().map(EmuHandle::state) {
            Some(CoreState::Loading) => {}
            Some(CoreState::Ready) => {
                self.reloading = false;
                self.app.link_reload_done();
            }
            Some(CoreState::Failed) | None => {
                self.reloading = false;
                self.emu = None;
                self.app.link_reload_failed();
                if let Some((stem, serial)) = self.app.take_link_reload() {
                    self.reload_for_link(&stem, serial);
                }
            }
        }
    }
}

pub(crate) fn trace() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("CARTRIDGE_TRACE").is_some())
}
