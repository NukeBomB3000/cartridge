//! Bluetooth: what the radio reports, and the jobs that drive `System/bluetooth.sh`.
//!
//! The script does the talking to bluetoothd. Its output is read here, off the frame loop: every
//! job runs on one worker thread, and what it learns comes back as news the app takes each tick.

use std::path::{Path, PathBuf};
#[cfg(feature = "device")]
use std::process::Stdio;
#[cfg(feature = "device")]
use std::sync::mpsc::{channel, Receiver, Sender};

use cartridge_input::Millis;

pub use cartridge_ui::kit::list_row::DeviceKind;

/// While the Bluetooth screen or Settings, which names the device connected, is open.
pub const REFRESH_MS: Millis = 2_000;
/// How long one look for devices lasts, X on the Bluetooth screen to its discovery turned off.
/// The controller looks over LE for 5.12 s, then over classic Bluetooth for 5.12 s, and only then
/// asks the classic devices it found their names: one round, and the names after it. On the
/// handheld every device in reach turned up within 2 s, and a TV's name came between 10 and 12 s
/// in, so a look much shorter would miss it. The script stops a discovery by itself after 20 s,
/// should the frontend be gone.
pub const LOOK_MS: Millis = 12_000;

/// The tools the script drives, each looked for in the system's own folders.
const TOOLS: [&str; 4] = ["rtk_hciattach", "hciconfig", "bluetoothctl", "systemctl"];
#[cfg(feature = "device")]
const BIN_DIRS: [&str; 4] = ["/usr/bin", "/usr/sbin", "/bin", "/sbin"];
/// Where bluetoothd is, which the system starts rather than finds on the path.
#[cfg(feature = "device")]
const DAEMON_DIRS: [&str; 2] = ["/usr/libexec/bluetooth", "/usr/lib/bluetooth"];

/// A device the radio knows: one paired, or one found by discovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    pub mac: String,
    /// Its name, in its own case: the one it gave, else the alias it was given. Empty for a
    /// paired device with neither; the script leaves out a device only found with neither.
    pub name: String,
    pub kind: DeviceKind,
    pub paired: bool,
    /// It may connect again by itself, without being asked.
    pub trusted: bool,
    pub connected: bool,
}

impl Device {
    /// The device as a list or a note names it: its name, or its address when it has none.
    pub fn shown(&self) -> &str {
        if self.name.is_empty() {
            &self.mac
        } else {
            &self.name
        }
    }
}

/// What `bluetooth.sh status` says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    /// Bluetooth is wanted, whether or not the radio has come up yet.
    pub enabled: bool,
    /// The radio is up and its controller powered.
    pub powered: bool,
    pub discovering: bool,
    pub devices: Vec<Device>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BluetoothJob {
    Up,
    Down,
    Refresh,
    /// Discovery on or off.
    Scan(bool),
    /// Pairs with a device found, trusts it and connects it.
    Pair(String),
    Connect(String),
    Disconnect(String),
    Forget(String),
}

/// Why a job for a device went wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fail {
    /// Bluetooth was not up.
    Radio,
    /// The device would not pair.
    Pair,
    /// The device is paired, but would not connect.
    Connect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BluetoothNews {
    /// `Up` finished: false when the radio never came up.
    Started(bool),
    Status(Status),
    /// A job for a device finished: a pair, a connect, a disconnect or a forget.
    Done {
        job: BluetoothJob,
        result: Result<(), Fail>,
    },
}

impl BluetoothJob {
    /// The device the job is for, when it is for one.
    pub fn device(&self) -> Option<&str> {
        match self {
            BluetoothJob::Pair(mac)
            | BluetoothJob::Connect(mac)
            | BluetoothJob::Disconnect(mac)
            | BluetoothJob::Forget(mac) => Some(mac),
            BluetoothJob::Up
            | BluetoothJob::Down
            | BluetoothJob::Refresh
            | BluetoothJob::Scan(_) => None,
        }
    }
}

pub trait BluetoothJobs: Send {
    /// The handheld has Bluetooth.
    fn present(&self) -> bool;

    fn ask(&mut self, job: BluetoothJob);

    fn news(&mut self) -> Vec<BluetoothNews>;
}

/// A device's kind, as the script names it from the icon bluetoothd gives it.
pub fn kind_of(word: &str) -> DeviceKind {
    match word {
        "audio" => DeviceKind::Audio,
        "gamepad" => DeviceKind::Gamepad,
        "keyboard" => DeviceKind::Keyboard,
        _ => DeviceKind::Other,
    }
}

/// A `device=` line of `bluetooth.sh status`: the address, the kind, whether it is paired,
/// trusted and connected, and the name, a tab between each.
fn parse_device(fields: &str) -> Option<Device> {
    let f: Vec<&str> = fields.splitn(6, '\t').collect();
    let [mac, kind, paired, trusted, connected, name] = f[..] else {
        return None;
    };
    let mac_ok = mac.len() == 17
        && mac.split(':').count() == 6
        && mac
            .split(':')
            .all(|b| b.len() == 2 && b.bytes().all(|c| c.is_ascii_hexdigit()));
    mac_ok.then(|| Device {
        mac: mac.to_ascii_uppercase(),
        name: name
            .chars()
            .map(|c| if c.is_control() { '\u{fffd}' } else { c })
            .collect(),
        kind: kind_of(kind),
        paired: paired == "yes",
        trusted: trusted == "yes",
        connected: connected == "yes",
    })
}

pub fn parse_status(text: &str) -> Status {
    let mut s = Status::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "enabled" => s.enabled = value == "yes",
            "powered" => s.powered = value == "yes",
            "discovering" => s.discovering = value == "yes",
            "device" => {
                if let Some(d) = parse_device(value) {
                    if !s.devices.iter().any(|known| known.mac == d.mac) {
                        s.devices.push(d);
                    }
                }
            }
            _ => {}
        }
    }
    s
}

/// Whether a handheld has Bluetooth, from its rfkill switches under `rfkill` and the folders its
/// tools are looked for in: a switch of type bluetooth, bluetoothd in one of `daemon_dirs`, and
/// every tool the script drives in one of `bin_dirs`.
pub fn radio_in(rfkill: &Path, bin_dirs: &[PathBuf], daemon_dirs: &[PathBuf]) -> bool {
    let switch = std::fs::read_dir(rfkill)
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            std::fs::read_to_string(e.path().join("type")).is_ok_and(|t| t.trim() == "bluetooth")
        });
    let found = |dirs: &[PathBuf], tool: &str| dirs.iter().any(|d| d.join(tool).is_file());
    switch && found(daemon_dirs, "bluetoothd") && TOOLS.iter().all(|tool| found(bin_dirs, tool))
}

pub fn bluetooth_jobs() -> Box<dyn BluetoothJobs> {
    #[cfg(feature = "device")]
    {
        Box::new(ScriptBluetooth::spawn(card_root()))
    }
    #[cfg(not(feature = "device"))]
    {
        Box::new(SimBluetooth::default())
    }
}

#[cfg(feature = "device")]
fn card_root() -> PathBuf {
    std::env::var_os("CARTRIDGE_ROOT")
        .unwrap_or_else(|| "/mnt/sdcard".into())
        .into()
}

/// The handheld's radio, through the card's script.
#[cfg(feature = "device")]
pub struct ScriptBluetooth {
    present: bool,
    jobs: Sender<BluetoothJob>,
    news: Receiver<BluetoothNews>,
}

#[cfg(feature = "device")]
impl ScriptBluetooth {
    pub fn spawn(root: PathBuf) -> Self {
        let (jobs, rx) = channel::<BluetoothJob>();
        let (tx, news) = channel::<BluetoothNews>();
        std::thread::spawn(move || {
            for job in rx {
                for n in work(&root, job) {
                    if tx.send(n).is_err() {
                        return;
                    }
                }
            }
        });
        // BaseOS says which images have Bluetooth; where it does not say, the radio's switch
        // and the tools to drive it do.
        let said = cartridge_power::baseos_release().and_then(|r| r.bluetooth);
        let dirs = |list: &[&str]| list.iter().map(PathBuf::from).collect::<Vec<_>>();
        let present = said != Some(false)
            && radio_in(
                Path::new("/sys/class/rfkill"),
                &dirs(&BIN_DIRS),
                &dirs(&DAEMON_DIRS),
            );
        ScriptBluetooth {
            present,
            jobs,
            news,
        }
    }
}

#[cfg(feature = "device")]
impl BluetoothJobs for ScriptBluetooth {
    fn present(&self) -> bool {
        self.present
    }

    fn ask(&mut self, job: BluetoothJob) {
        let _ = self.jobs.send(job);
    }

    fn news(&mut self) -> Vec<BluetoothNews> {
        self.news.try_iter().collect()
    }
}

/// The command that runs the card's script with `args`.
pub fn script(root: &Path, args: &[&str]) -> std::process::Command {
    let mut cmd = std::process::Command::new("/bin/sh");
    cmd.arg(root.join("System/bluetooth.sh")).args(args);
    // what it leaves running, rtk_hciattach, bluetoothd and a scan, starts as BaseOS's own
    // daemons do
    crate::spawn::as_a_daemon(&mut cmd);
    cmd
}

/// The news a device job's exit code makes: 0 done, 3 not paired, 4 paired but not connected,
/// and anything else Bluetooth not being up. A connect that fails is a connection that failed,
/// whatever the code.
pub fn done(job: &BluetoothJob, code: Option<i32>) -> Option<BluetoothNews> {
    job.device()?;
    let result = match code {
        Some(0) => Ok(()),
        _ if matches!(job, BluetoothJob::Connect(_)) => Err(Fail::Connect),
        Some(3) => Err(Fail::Pair),
        Some(4) => Err(Fail::Connect),
        _ => Err(Fail::Radio),
    };
    Some(BluetoothNews::Done {
        job: job.clone(),
        result,
    })
}

#[cfg(feature = "device")]
fn output(root: &Path, args: &[&str]) -> Option<String> {
    let out = script(root, args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(feature = "device")]
fn status_code(root: &Path, args: &[&str]) -> Option<i32> {
    script(root, args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?
        .code()
}

#[cfg(feature = "device")]
fn work(root: &Path, job: BluetoothJob) -> Vec<BluetoothNews> {
    let mut news = match &job {
        BluetoothJob::Up => vec![BluetoothNews::Started(
            status_code(root, &["up"]) == Some(0),
        )],
        BluetoothJob::Down => {
            status_code(root, &["down"]);
            Vec::new()
        }
        BluetoothJob::Refresh => Vec::new(),
        // the status after discovery is turned off has the names that came in last
        BluetoothJob::Scan(on) => {
            status_code(root, &["scan", if *on { "on" } else { "off" }]);
            Vec::new()
        }
        BluetoothJob::Pair(mac)
        | BluetoothJob::Connect(mac)
        | BluetoothJob::Disconnect(mac)
        | BluetoothJob::Forget(mac) => {
            let verb = match job {
                BluetoothJob::Pair(_) => "pair",
                BluetoothJob::Connect(_) => "connect",
                BluetoothJob::Disconnect(_) => "disconnect",
                _ => "forget",
            };
            done(&job, status_code(root, &[verb, mac]))
                .into_iter()
                .collect()
        }
    };
    if let Some(text) = output(root, &["status"]) {
        news.push(BluetoothNews::Status(parse_status(&text)));
    }
    news
}

/// A radio for the desktop and the tests, with flow.html's devices in reach: two paired, the
/// gamepad connecting by itself whenever Bluetooth comes up, and two more found at once while
/// discovering and forgotten once it stops, as bluetoothd forgets them soon after. A pair or a
/// connect takes two refreshes, so the screen shows it under way; a device forgotten is gone for
/// good.
pub struct SimBluetooth {
    enabled: bool,
    discovering: bool,
    devices: Vec<Device>,
    /// A pair or a connect under way, and how many refreshes more it takes.
    pending: Option<(BluetoothJob, u8)>,
    news: Vec<BluetoothNews>,
}

impl Default for SimBluetooth {
    fn default() -> Self {
        let device = |mac: &str, name: &str, kind, paired| Device {
            mac: mac.into(),
            name: name.into(),
            kind,
            paired,
            trusted: paired,
            connected: false,
        };
        SimBluetooth {
            enabled: false,
            discovering: false,
            devices: vec![
                device(
                    SimBluetooth::GAMEPAD,
                    "8BitDo SN30 Pro",
                    DeviceKind::Gamepad,
                    true,
                ),
                device("38:18:4C:AB:CD:EF", "WH-1000XM4", DeviceKind::Audio, true),
                device(
                    "DC:2C:26:01:02:03",
                    "Keychron K3",
                    DeviceKind::Keyboard,
                    false,
                ),
                device(
                    "C8:0F:10:4D:7A:25",
                    "Mi Smart Band 4",
                    DeviceKind::Other,
                    false,
                ),
            ],
            pending: None,
            news: Vec::new(),
        }
    }
}

impl SimBluetooth {
    /// The gamepad, which connects by itself as Bluetooth comes up.
    pub const GAMEPAD: &'static str = "E4:17:D8:9A:01:22";

    fn device(&mut self, mac: &str) -> Option<&mut Device> {
        self.devices.iter_mut().find(|d| d.mac == mac)
    }

    fn status(&self) -> Status {
        Status {
            enabled: self.enabled,
            powered: self.enabled,
            discovering: self.discovering,
            devices: self
                .devices
                .iter()
                .filter(|d| self.enabled && (d.paired || self.discovering))
                .cloned()
                .collect(),
        }
    }

    /// Settles the pair or connect under way once it has taken its refreshes.
    fn settle(&mut self) {
        let Some((job, left)) = self.pending.take() else {
            return;
        };
        if left > 1 {
            self.pending = Some((job, left - 1));
            return;
        }
        if let BluetoothJob::Pair(mac) | BluetoothJob::Connect(mac) = &job {
            if let Some(d) = self.device(mac) {
                d.paired = true;
                d.trusted = true;
                d.connected = true;
            }
        }
        self.news.push(BluetoothNews::Done {
            job,
            result: Ok(()),
        });
    }
}

impl BluetoothJobs for SimBluetooth {
    fn present(&self) -> bool {
        true
    }

    fn ask(&mut self, job: BluetoothJob) {
        match job {
            BluetoothJob::Up => {
                self.enabled = true;
                if let Some(pad) = self.device(SimBluetooth::GAMEPAD) {
                    pad.connected = pad.paired;
                }
                self.news.push(BluetoothNews::Started(true));
            }
            BluetoothJob::Down => {
                self.enabled = false;
                self.discovering = false;
                self.pending = None;
                for d in &mut self.devices {
                    d.connected = false;
                }
            }
            BluetoothJob::Refresh => self.settle(),
            // the devices in reach are found at once
            BluetoothJob::Scan(on) => self.discovering = on && self.enabled,
            BluetoothJob::Pair(_) | BluetoothJob::Connect(_) => {
                self.pending = Some((job, 2));
                return;
            }
            BluetoothJob::Disconnect(ref mac) => {
                if let Some(d) = self.device(mac) {
                    d.connected = false;
                }
                self.news.push(BluetoothNews::Done {
                    job,
                    result: Ok(()),
                });
            }
            BluetoothJob::Forget(ref mac) => {
                self.devices.retain(|d| d.mac != *mac);
                self.news.push(BluetoothNews::Done {
                    job,
                    result: Ok(()),
                });
            }
        }
        let status = self.status();
        self.news.push(BluetoothNews::Status(status));
    }

    fn news(&mut self) -> Vec<BluetoothNews> {
        std::mem::take(&mut self.news)
    }
}
