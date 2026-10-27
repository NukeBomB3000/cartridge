//! What cartridge reads of `System/bluetooth.sh`: its status, the exit codes of the jobs for a
//! device, whether a handheld has Bluetooth at all, and the desktop's stand-in radio.

use std::path::{Path, PathBuf};

use cartridge::bluetooth::{
    done, kind_of, parse_status, radio_in, script, BluetoothJob, BluetoothJobs, BluetoothNews,
    Device, DeviceKind, Fail, SimBluetooth, Status,
};

fn device(mac: &str, name: &str, kind: DeviceKind, paired: bool, connected: bool) -> Device {
    Device {
        mac: mac.into(),
        name: name.into(),
        kind,
        paired,
        trusted: paired,
        connected,
    }
}

#[test]
fn status_reads_the_switch_the_radio_and_each_device_line() {
    let text = "enabled=yes\npowered=yes\ndiscovering=yes\n\
        device=E4:17:D8:9A:01:22\tgamepad\tyes\tyes\tyes\t8BitDo SN30 Pro\n\
        device=38:18:4C:AB:CD:EF\taudio\tyes\tno\tno\tWH-1000XM4\n\
        device=DC:2C:26:01:02:03\tkeyboard\tno\tno\tno\tKeychron K3\n\
        device=4C:4C:6B:1A:80:D3\tother\tno\tno\tno\t\n";
    let s = parse_status(text);
    assert!(s.enabled && s.powered && s.discovering);
    let mut cans = device(
        "38:18:4C:AB:CD:EF",
        "WH-1000XM4",
        DeviceKind::Audio,
        true,
        false,
    );
    cans.trusted = false;
    assert_eq!(
        s.devices,
        [
            device(
                "E4:17:D8:9A:01:22",
                "8BitDo SN30 Pro",
                DeviceKind::Gamepad,
                true,
                true
            ),
            cans,
            device(
                "DC:2C:26:01:02:03",
                "Keychron K3",
                DeviceKind::Keyboard,
                false,
                false
            ),
            device("4C:4C:6B:1A:80:D3", "", DeviceKind::Other, false, false),
        ]
    );
    assert_eq!(
        s.devices[3].shown(),
        "4C:4C:6B:1A:80:D3",
        "a nameless device"
    );
    assert_eq!(s.devices[0].shown(), "8BitDo SN30 Pro");
}

#[test]
fn status_of_a_radio_that_is_down_has_no_devices() {
    let s = parse_status("enabled=no\npowered=no\ndiscovering=no\n");
    assert_eq!(s, Status::default());
    let s = parse_status("enabled=yes\npowered=no\ndiscovering=no\n");
    assert!(s.enabled && !s.powered, "wanted but still coming up");
}

#[test]
fn a_name_keeps_its_tabs_and_case_and_loses_only_what_cannot_be_printed() {
    let s = parse_status("device=aa:bb:cc:dd:ee:ff\taudio\tno\tno\tno\tMy\tCafé \u{7}Speaker\n");
    assert_eq!(s.devices.len(), 1);
    assert_eq!(
        s.devices[0].mac, "AA:BB:CC:DD:EE:FF",
        "the address in capitals"
    );
    assert_eq!(s.devices[0].name, "My\u{fffd}Café \u{fffd}Speaker");
}

#[test]
fn a_line_that_is_no_device_is_passed_over() {
    for line in [
        "device=nope\tgamepad\tyes\tyes\tyes\tPad",
        "device=E4:17:D8:9A:01\tgamepad\tyes\tyes\tyes\tPad",
        "device=E4:17:D8:9A:01:2G\tgamepad\tyes\tyes\tyes\tPad",
        "device=E4:17:D8:9A:01:22\tgamepad\tyes",
        "device=",
        "E4:17:D8:9A:01:22\tgamepad\tyes\tyes\tyes\tPad",
    ] {
        assert!(parse_status(line).devices.is_empty(), "{line:?} was read");
    }
    let twice = "device=E4:17:D8:9A:01:22\tgamepad\tyes\tyes\tyes\tPad\n\
        device=E4:17:D8:9A:01:22\tgamepad\tyes\tyes\tyes\tPad\n";
    assert_eq!(
        parse_status(twice).devices.len(),
        1,
        "one device listed twice"
    );
}

#[test]
fn the_kinds_are_the_scripts_words_and_anything_else_is_other() {
    assert_eq!(kind_of("audio"), DeviceKind::Audio);
    assert_eq!(kind_of("gamepad"), DeviceKind::Gamepad);
    assert_eq!(kind_of("keyboard"), DeviceKind::Keyboard);
    for word in ["other", "", "phone", "Audio", "input-gaming"] {
        assert_eq!(kind_of(word), DeviceKind::Other, "{word:?}");
    }
}

#[test]
fn a_device_jobs_exit_code_says_how_it_went() {
    let mac = "E4:17:D8:9A:01:22".to_string();
    let news = |job: BluetoothJob, code| done(&job, code);
    let result =
        |job: BluetoothJob, r: Result<(), Fail>| Some(BluetoothNews::Done { job, result: r });
    let pair = || BluetoothJob::Pair(mac.clone());
    let connect = || BluetoothJob::Connect(mac.clone());
    assert_eq!(news(pair(), Some(0)), result(pair(), Ok(())));
    assert_eq!(news(pair(), Some(3)), result(pair(), Err(Fail::Pair)));
    assert_eq!(news(pair(), Some(4)), result(pair(), Err(Fail::Connect)));
    assert_eq!(news(pair(), Some(1)), result(pair(), Err(Fail::Radio)));
    assert_eq!(
        news(pair(), None),
        result(pair(), Err(Fail::Radio)),
        "killed"
    );
    assert_eq!(news(connect(), Some(0)), result(connect(), Ok(())));
    for code in [Some(1), Some(3), Some(4), None] {
        assert_eq!(
            news(connect(), code),
            result(connect(), Err(Fail::Connect)),
            "{code:?}"
        );
    }
    let disconnect = BluetoothJob::Disconnect(mac.clone());
    assert_eq!(
        news(disconnect.clone(), Some(1)),
        result(disconnect, Err(Fail::Radio))
    );
    let forget = BluetoothJob::Forget(mac.clone());
    assert_eq!(news(forget.clone(), Some(0)), result(forget, Ok(())));
    assert_eq!(pair().device(), Some(mac.as_str()));
    for job in [
        BluetoothJob::Up,
        BluetoothJob::Down,
        BluetoothJob::Refresh,
        BluetoothJob::Scan(true),
    ] {
        assert_eq!(done(&job, Some(0)), None, "{job:?} is no job for a device");
    }
}

#[test]
fn a_job_runs_the_cards_script_through_sh() {
    let d = tempfile::tempdir().unwrap();
    let cmd = script(d.path(), &["pair", "E4:17:D8:9A:01:22"]);
    let argv: Vec<String> = std::iter::once(cmd.get_program())
        .chain(cmd.get_args())
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let at = d.path().join("System/bluetooth.sh");
    assert_eq!(
        argv,
        ["/bin/sh", at.to_str().unwrap(), "pair", "E4:17:D8:9A:01:22"]
    );
}

/// A system with the rfkill switches `types`, and the tools in `tools`, bluetoothd among them.
struct System {
    d: tempfile::TempDir,
}

impl System {
    fn new(types: &[&str], tools: &[&str]) -> System {
        let d = tempfile::tempdir().unwrap();
        for (i, t) in types.iter().enumerate() {
            let at = d.path().join(format!("rfkill/rfkill{i}"));
            std::fs::create_dir_all(&at).unwrap();
            std::fs::write(at.join("type"), format!("{t}\n")).unwrap();
        }
        std::fs::create_dir_all(d.path().join("rfkill")).unwrap();
        for dir in ["usr/bin", "usr/libexec/bluetooth"] {
            std::fs::create_dir_all(d.path().join(dir)).unwrap();
        }
        for tool in tools {
            let dir = if *tool == "bluetoothd" {
                "usr/libexec/bluetooth"
            } else {
                "usr/bin"
            };
            std::fs::write(d.path().join(dir).join(tool), "").unwrap();
        }
        System { d }
    }

    fn at(&self, p: &str) -> PathBuf {
        self.d.path().join(p)
    }

    fn has_bluetooth(&self) -> bool {
        radio_in(
            &self.at("rfkill"),
            &[self.at("usr/bin")],
            &[self.at("usr/libexec/bluetooth")],
        )
    }
}

const ALL_TOOLS: [&str; 5] = [
    "rtk_hciattach",
    "hciconfig",
    "bluetoothctl",
    "systemctl",
    "bluetoothd",
];

#[test]
fn a_handheld_has_bluetooth_with_its_switch_and_every_tool() {
    assert!(System::new(&["wlan", "bluetooth", "wlan"], &ALL_TOOLS).has_bluetooth());
}

#[test]
fn without_a_bluetooth_switch_or_any_one_tool_there_is_no_bluetooth() {
    assert!(
        !System::new(&["wlan", "wlan"], &ALL_TOOLS).has_bluetooth(),
        "Wi-Fi's switches only"
    );
    assert!(!System::new(&[], &ALL_TOOLS).has_bluetooth(), "no switches");
    for missing in ALL_TOOLS {
        let tools: Vec<&str> = ALL_TOOLS.into_iter().filter(|t| *t != missing).collect();
        assert!(
            !System::new(&["bluetooth"], &tools).has_bluetooth(),
            "Bluetooth without {missing}"
        );
    }
    assert!(!radio_in(Path::new("/nonexistent/rfkill"), &[], &[]));
}

#[test]
fn the_desktops_radio_holds_flows_devices() {
    let mut sim = SimBluetooth::default();
    assert!(sim.present());
    sim.ask(BluetoothJob::Refresh);
    let off = sim.news();
    assert_eq!(
        off.last(),
        Some(&BluetoothNews::Status(Status::default())),
        "devices while Bluetooth is off"
    );
    sim.ask(BluetoothJob::Up);
    let news = sim.news();
    assert_eq!(news[0], BluetoothNews::Started(true));
    let BluetoothNews::Status(s) = &news[1] else {
        panic!("no status after up: {news:?}");
    };
    let names: Vec<&str> = s.devices.iter().map(Device::shown).collect();
    assert_eq!(
        names,
        ["8BitDo SN30 Pro", "WH-1000XM4"],
        "only the paired ones"
    );
    assert!(
        s.devices[0].connected,
        "the gamepad did not connect by itself"
    );
    assert!(!s.devices[1].connected);

    sim.ask(BluetoothJob::Scan(true));
    sim.ask(BluetoothJob::Refresh);
    let Some(BluetoothNews::Status(s)) = sim.news().pop() else {
        panic!("no status");
    };
    let names: Vec<&str> = s.devices.iter().map(Device::shown).collect();
    assert_eq!(
        names,
        [
            "8BitDo SN30 Pro",
            "WH-1000XM4",
            "Keychron K3",
            "Mi Smart Band 4"
        ]
    );
    assert_eq!(
        s.devices.iter().map(|d| d.kind).collect::<Vec<_>>(),
        [
            DeviceKind::Gamepad,
            DeviceKind::Audio,
            DeviceKind::Keyboard,
            DeviceKind::Other
        ]
    );

    // the devices only found are forgotten once discovery stops, as bluetoothd forgets them
    sim.ask(BluetoothJob::Scan(false));
    let Some(BluetoothNews::Status(s)) = sim.news().pop() else {
        panic!("no status after discovery stopped");
    };
    let names: Vec<&str> = s.devices.iter().map(Device::shown).collect();
    assert_eq!(names, ["8BitDo SN30 Pro", "WH-1000XM4"]);
}

#[test]
fn the_desktops_radio_lets_a_device_forgotten_go() {
    let mut sim = SimBluetooth::default();
    sim.ask(BluetoothJob::Up);
    sim.ask(BluetoothJob::Scan(true));
    sim.ask(BluetoothJob::Forget("38:18:4C:AB:CD:EF".into()));
    let Some(BluetoothNews::Status(s)) = sim.news().pop() else {
        panic!("no status");
    };
    assert!(
        s.devices.iter().all(|d| d.name != "WH-1000XM4"),
        "{:?}",
        s.devices
    );
}

#[test]
fn the_desktops_radio_takes_two_refreshes_to_pair() {
    let mut sim = SimBluetooth::default();
    sim.ask(BluetoothJob::Up);
    sim.ask(BluetoothJob::Scan(true));
    sim.news();
    let keys = "DC:2C:26:01:02:03".to_string();
    sim.ask(BluetoothJob::Pair(keys.clone()));
    assert_eq!(sim.news(), [], "the pairing answered at once");
    sim.ask(BluetoothJob::Refresh);
    assert!(
        !sim.news()
            .iter()
            .any(|n| matches!(n, BluetoothNews::Done { .. })),
        "done after one refresh"
    );
    sim.ask(BluetoothJob::Refresh);
    let news = sim.news();
    assert!(news.contains(&BluetoothNews::Done {
        job: BluetoothJob::Pair(keys.clone()),
        result: Ok(())
    }));
    let Some(BluetoothNews::Status(s)) = news.last() else {
        panic!("no status: {news:?}");
    };
    let k = s.devices.iter().find(|d| d.mac == keys).unwrap();
    assert!(k.paired && k.trusted && k.connected, "{k:?}");
}
