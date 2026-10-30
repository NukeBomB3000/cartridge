use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cartridge_input::{Btn, InputSource, Millis, RawEvent};

use super::evdev::{
    decode, device_name, layout_of, pick_devices, to_raw_for, Ev, Hat, Layout, PadAxes, Range,
    ABS_RX, ABS_RY, ABS_RZ, ABS_X, ABS_Y, ABS_Z, EVENT_BYTES, EV_ABS, EV_KEY, EV_SYN,
};
use super::trace;

const DEV: &str = "/dev/input";
const SYS: &str = "/sys/class/input";
const PSY: &str = "/sys/class/power_supply";

const LID_POLL_MS: Millis = 150;

pub struct DeviceInput {
    pending: Arc<Mutex<Vec<RawEvent>>>,
    hall: Option<PathBuf>,
    lid_shut: Option<bool>,
    next_lid_poll: Millis,
}

impl DeviceInput {
    pub fn open(root: &Path) -> Self {
        DeviceInput::open_in(Path::new(DEV), Path::new(SYS), root, trace::enabled())
    }

    pub fn open_in(dev: &Path, sys: &Path, root: &Path, trace: bool) -> Self {
        let trace = trace.then(|| Trace::start(root, dev, sys)).flatten();
        DeviceInput::open_traced(dev, sys, trace)
    }

    fn open_traced(dev: &Path, sys: &Path, trace: Option<Arc<Trace>>) -> Self {
        let pending: Arc<Mutex<Vec<RawEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let opened: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
        let nodes = Nodes {
            dev: dev.to_path_buf(),
            sys: sys.to_path_buf(),
            pending: pending.clone(),
            opened,
            trace,
        };
        nodes.open_new();
        // a gamepad or a pair of headphones that connects later brings an input node of its own
        let watch = std::thread::Builder::new()
            .name("cartridge-input-watch".into())
            .spawn(move || loop {
                std::thread::sleep(WATCH_EVERY);
                nodes.open_new();
            });
        if let Err(e) = watch {
            eprintln!("cartridge: input watch: {e}");
        }
        DeviceInput {
            pending,
            hall: find_hall(Path::new(PSY)),
            lid_shut: None,
            next_lid_poll: 0,
        }
    }
}

/// How often `/dev/input` is looked at for a node that was not there before.
const WATCH_EVERY: Duration = Duration::from_secs(1);

struct Nodes {
    dev: PathBuf,
    sys: PathBuf,
    pending: Arc<Mutex<Vec<RawEvent>>>,
    opened: Arc<Mutex<HashSet<PathBuf>>>,
    trace: Option<Arc<Trace>>,
}

impl Nodes {
    /// Starts a reader for each wanted node that has none.
    fn open_new(&self) {
        for node in pick_devices(&self.dev, &self.sys) {
            if !self
                .opened
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(node.clone())
            {
                continue;
            }
            let layout = layout_of(&self.sys, &node);
            eprintln!(
                "cartridge: input {} ({}){}",
                node.display(),
                device_name(&self.sys, &node),
                if layout == Layout::Pad {
                    " as a pad"
                } else {
                    ""
                }
            );
            quicken_poll(&self.sys, &node);
            let queue = self.pending.clone();
            let name = node.display().to_string();
            let trace = self.trace.clone();
            let opened = self.opened.clone();
            let at = node.clone();
            let spawned = std::thread::Builder::new()
                .name(format!("cartridge-input-{name}"))
                .spawn(move || {
                    read_node(&node, layout, &queue, trace.as_deref());
                    // gone, or never opened: a node of the same name later is a new device
                    opened.lock().unwrap_or_else(|e| e.into_inner()).remove(&at);
                });
            if let Err(e) = spawned {
                eprintln!("cartridge: input {name}: {e}");
            }
        }
    }
}

const POLL_MS: u32 = 10;

fn quicken_poll(sys: &Path, node: &Path) {
    let Some(name) = node.file_name() else {
        return;
    };
    let attr = sys.join(name).join("device/poll");
    let Some(now) = std::fs::read_to_string(&attr)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
    else {
        return;
    };
    if now > POLL_MS {
        match std::fs::write(&attr, POLL_MS.to_string()) {
            Ok(()) => eprintln!(
                "cartridge: input {}: poll {now} ms -> {POLL_MS} ms",
                node.display()
            ),
            Err(e) => eprintln!("cartridge: input {}: poll: {e}", node.display()),
        }
    }
}

/// How far axis `code` of the open node goes, as its driver says (EVIOCGABS), else `default`.
fn axis_range(file: &File, code: u16, default: Range) -> Range {
    let mut info = [0i32; 6];
    let request = 0x8018_4540u64 + u64::from(code);
    // SAFETY: EVIOCGABS fills a struct input_absinfo, six 32-bit integers, and nothing else
    let ok = unsafe { libc::ioctl(file.as_raw_fd(), request as _, info.as_mut_ptr()) } == 0;
    if ok && info[2] > info[1] {
        Range {
            min: info[1],
            max: info[2],
        }
    } else {
        default
    }
}

fn read_node(node: &Path, layout: Layout, queue: &Mutex<Vec<RawEvent>>, trace: Option<&Trace>) {
    let label = node
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| node.display().to_string());
    let mut hat = Hat::default();
    let mut file = match File::open(node) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("cartridge: input {}: {e}", node.display());
            return;
        }
    };
    let mut axes = PadAxes::new(
        [ABS_X, ABS_Y, ABS_RX, ABS_RY].map(|c| axis_range(&file, c, Range::STICK)),
        [ABS_Z, ABS_RZ].map(|c| axis_range(&file, c, Range::TRIGGER)),
    );
    let mut buf = [0u8; EVENT_BYTES * 16];
    loop {
        let read = match file.read(&mut buf) {
            Ok(0) => return,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("cartridge: input {}: {e}", node.display());
                return;
            }
        };
        let mut events = Vec::new();
        for ev in buf[..read].chunks_exact(EVENT_BYTES).filter_map(decode) {
            if let Some(trace) = trace {
                trace.event(&label, ev);
            }
            if ev.kind == EV_KEY && ev.value == 1 {
                crate::latency::read();
            }
            match ev.kind {
                EV_ABS => {
                    let pressed = if layout == Layout::Pad {
                        axes.feed(&mut hat, ev)
                    } else {
                        hat.feed(ev)
                    };
                    if pressed.iter().any(|e| matches!(e, RawEvent::Down(_))) {
                        crate::latency::read();
                    }
                    events.extend(pressed);
                }
                _ => events.extend(to_raw_for(layout, ev)),
            }
        }
        if events.is_empty() {
            continue;
        }
        queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(events);
    }
}

struct Trace {
    out: Mutex<File>,
    began: Instant,
}

impl Trace {
    fn start(root: &Path, dev: &Path, sys: &Path) -> Option<Arc<Trace>> {
        let path = root.join(trace::TRACE_FILE);
        let mut file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("cartridge: {}: {e}", path.display());
                return None;
            }
        };
        for line in trace::survey(dev, sys) {
            let _ = writeln!(file, "{line}");
        }
        let _ = writeln!(file, "--");
        let _ = file.flush();
        Some(Arc::new(Trace {
            out: Mutex::new(file),
            began: Instant::now(),
        }))
    }

    fn event(&self, node: &str, ev: Ev) {
        if ev.kind == EV_SYN {
            return;
        }
        let line = trace::event_line(node, self.began.elapsed().as_millis(), ev);
        let mut out = self.out.lock().unwrap_or_else(|e| e.into_inner());
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    }
}

impl DeviceInput {
    fn read_lid(&self) -> Option<bool> {
        let raw = std::fs::read_to_string(self.hall.as_ref()?).ok()?;
        match raw.trim() {
            "0" => Some(true),
            "1" => Some(false),
            _ => None,
        }
    }
}

impl InputSource for DeviceInput {
    fn poll(&mut self, now: Millis) -> Vec<RawEvent> {
        let mut out = std::mem::take(&mut *self.pending.lock().unwrap_or_else(|e| e.into_inner()));
        if self.hall.is_some() && now >= self.next_lid_poll {
            self.next_lid_poll = now + LID_POLL_MS;
            if let Some(shut) = self.read_lid() {
                if self.lid_shut != Some(shut) {
                    self.lid_shut = Some(shut);
                    out.push(if shut {
                        RawEvent::Down(Btn::Lid)
                    } else {
                        RawEvent::Up(Btn::Lid)
                    });
                }
            }
        }
        out
    }
}

fn find_hall(psy: &Path) -> Option<PathBuf> {
    let mut supplies: Vec<PathBuf> = std::fs::read_dir(psy)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    supplies.sort();
    supplies
        .into_iter()
        .map(|d| d.join("hallkey"))
        .find(|p| p.is_file())
}
