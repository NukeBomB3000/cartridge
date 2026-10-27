use cartridge::wifi::{
    bars, decode, hex, link_up, parse_saved, parse_scan, parse_status, passphrase_ok, shown,
    AskFail, Saved, SimWifi, WifiJob, WifiJobs, WifiNews,
};

#[test]
fn an_ssid_comes_back_byte_for_byte_from_wpa_cli_escapes() {
    assert_eq!(decode(r"Caf\xc3\xa9"), "Café".as_bytes());
    assert_eq!(decode(r#"say \"hi\""#), br#"say "hi""#);
    assert_eq!(decode(r"back\\slash"), br"back\slash");
    assert_eq!(decode(r"tab\there"), b"tab\there");
    assert_eq!(decode(r"\e\n\r"), b"\x1b\n\r");
    assert_eq!(
        decode(r"odd\x4"),
        br"odd\x4",
        "a short escape is kept as written"
    );
    assert_eq!(decode(r"end\"), br"end\");
}

#[test]
fn an_ssid_is_named_to_wpa_supplicant_in_hex() {
    assert_eq!(hex(b"Home"), "486f6d65");
    assert_eq!(hex("Café".as_bytes()), "436166c3a9");
    assert_eq!(hex(b""), "");
}

#[test]
fn an_ssid_that_is_not_text_is_still_shown() {
    assert_eq!(shown("Café".as_bytes()), "Café");
    assert_eq!(shown(b"a\x07b"), "a\u{fffd}b");
    assert_eq!(shown(&[0xff, b'x']), "\u{fffd}x");
}

#[test]
fn signal_is_four_bars_near_the_router_and_one_at_the_edge() {
    assert_eq!(bars(-40), 4);
    assert_eq!(bars(-55), 4);
    assert_eq!(bars(-60), 3);
    assert_eq!(bars(-70), 2);
    assert_eq!(bars(-90), 1);
}

#[test]
fn a_passphrase_is_eight_to_sixty_three_printable_characters() {
    assert!(!passphrase_ok("seven77"));
    assert!(passphrase_ok("eight888"));
    assert!(passphrase_ok("with spaces and ~!@#"));
    assert!(passphrase_ok(&"x".repeat(63)));
    assert!(!passphrase_ok(&"x".repeat(64)));
    assert!(
        !passphrase_ok("naïve passphrase"),
        "wpa_supplicant takes ASCII only"
    );
    assert!(!passphrase_ok("tab\tinside"));
}

const SCAN: &str = "bssid / frequency / signal level / flags / ssid
02:00:00:00:00:01\t2437\t-61\t[WPA2-PSK-CCMP][ESS]\tHome
02:00:00:00:00:02\t5180\t-48\t[WPA2-PSK-CCMP][ESS]\tHome
02:00:00:00:00:03\t2412\t-70\t[ESS]\tCaf\\xc3\\xa9
02:00:00:00:00:04\t2412\t-50\t[WPA2-PSK-CCMP][ESS]\t
02:00:00:00:00:05\t2412\t-52\t[WPA2-PSK-CCMP][ESS]\t\\x00\\x00\\x00
02:00:00:00:00:06\t2462\t-58\t[WPA2-EAP-CCMP][ESS]\teduroam
02:00:00:00:00:07\t2462\t-75\t[WPA2-EAP+PSK-CCMP][ESS]\tOffice
02:00:00:00:00:08\t2462\t-80\t[RSN-SAE-CCMP][ESS]\tNew Router
not a line
";

#[test]
fn a_scan_lists_each_network_once_at_its_strongest() {
    let seen = parse_scan(SCAN);
    let names: Vec<String> = seen.iter().map(|s| shown(&s.ssid)).collect();
    assert_eq!(names, ["Home", "Café", "Office", "New Router"]);
    assert_eq!(seen[0].rssi, -48, "Home was not taken at its strongest");
    assert!(seen[0].secured);
    assert!(
        !seen[1].secured,
        "an [ESS]-only network needs no passphrase"
    );
    assert!(seen[3].secured, "WPA3 needs a passphrase too");
}

#[test]
fn a_scan_leaves_out_hidden_networks_and_those_that_need_more_than_a_passphrase() {
    let seen = parse_scan(SCAN);
    assert!(!seen.iter().any(|s| s.ssid.is_empty() || s.ssid[0] == 0));
    assert!(!seen.iter().any(|s| s.ssid == b"eduroam"));
}

#[test]
fn saved_networks_carry_their_ids_and_whether_they_were_turned_away() {
    let saved = parse_saved(
        "network id / ssid / bssid / flags
0\tHome\tany\t[CURRENT]
1\tCaf\\xc3\\xa9\tany\t
2\tCabin\tany\t[DISABLED]
3\tOffice\tany\t[TEMP-DISABLED]
",
    );
    assert_eq!(
        saved,
        [
            Saved {
                id: 0,
                ssid: b"Home".to_vec(),
                current: true,
                refused: false
            },
            Saved {
                id: 1,
                ssid: "Café".as_bytes().to_vec(),
                current: false,
                refused: false
            },
            Saved {
                id: 2,
                ssid: b"Cabin".to_vec(),
                current: false,
                refused: false
            },
            Saved {
                id: 3,
                ssid: b"Office".to_vec(),
                current: false,
                refused: true
            },
        ]
    );
}

#[test]
fn status_reads_the_network_joined_its_address_and_network_time() {
    let s = parse_status(
        "enabled=yes
bssid=02:00:00:00:00:01
ssid=Caf\\xc3\\xa9
wpa_state=COMPLETED
ip_address=192.168.1.20
p2p_device_address=e6:5f:01:2a:9c:33
address=e4:5f:01:2a:9c:33
RSSI=-52
hostname=rg35xxsp
ntp=yes
ntp_synced=no
",
    );
    assert!(s.enabled && s.joined());
    assert_eq!(s.ssid.as_deref(), Some("Café".as_bytes()));
    assert_eq!(s.ip.as_deref(), Some("192.168.1.20"));
    assert_eq!(s.rssi, Some(-52));
    assert_eq!(s.mac.as_deref(), Some("e4:5f:01:2a:9c:33"));
    assert_eq!(s.hostname, "rg35xxsp");
    assert!(s.ntp && !s.ntp_synced);
}

#[test]
fn an_address_left_from_before_counts_only_while_joined() {
    let s = parse_status("enabled=yes\nwpa_state=SCANNING\nip_address=192.168.1.20\n");
    assert!(!s.joined());
    assert_eq!(s.ip, None);
    let off = parse_status("enabled=no\nhostname=rg35xxsp\nntp=\nntp_synced=\n");
    assert!(!off.enabled && !off.ntp);
    assert_eq!(off.state, "");
}

#[test]
fn a_glance_counts_the_link_joined_only_when_the_kernel_says_up() {
    assert!(link_up("up\n"));
    for not in ["dormant\n", "down\n", "unknown\n", "lowerlayerdown\n", ""] {
        assert!(!link_up(not), "{not:?} counted as joined");
    }
}

#[test]
fn the_desktop_radio_glances_without_saying_its_whole_status() {
    let mut sim = SimWifi::default();
    sim.ask(WifiJob::Glance);
    assert_eq!(sim.news(), [WifiNews::Glanced(false)]);
    sim.ask(WifiJob::Join {
        ssid: b"Home".to_vec(),
        psk: "long enough".into(),
        replace: None,
    });
    sim.news();
    sim.ask(WifiJob::Glance);
    assert_eq!(sim.news(), [WifiNews::Glanced(true)]);
}

fn statuses(news: &[WifiNews]) -> Vec<&cartridge::wifi::Status> {
    news.iter()
        .filter_map(|n| match n {
            WifiNews::Status(s) => Some(s),
            _ => None,
        })
        .collect()
}

#[test]
fn the_desktop_radio_hears_nothing_while_off() {
    let mut sim = SimWifi::default();
    sim.ask(WifiJob::Scan);
    assert_eq!(sim.news(), [WifiNews::Seen(Vec::new())]);
    sim.ask(WifiJob::Up);
    sim.ask(WifiJob::Scan);
    let news = sim.news();
    assert!(news.contains(&WifiNews::Started(true)));
    assert!(news
        .iter()
        .any(|n| matches!(n, WifiNews::Seen(s) if s.len() == SimWifi::NEARBY.len())));
}

#[test]
fn the_desktop_radio_joins_with_any_passphrase_wpa_supplicant_would_take() {
    let mut sim = SimWifi::default();
    sim.ask(WifiJob::Up);
    sim.news();
    sim.ask(WifiJob::Join {
        ssid: b"Home".to_vec(),
        psk: "short".into(),
        replace: None,
    });
    assert!(sim
        .news()
        .contains(&WifiNews::Asked(Err(AskFail::BadPassphrase))));
    sim.ask(WifiJob::Join {
        ssid: b"Home".to_vec(),
        psk: "long enough".into(),
        replace: None,
    });
    let news = sim.news();
    assert!(news.contains(&WifiNews::Asked(Ok(()))));
    let s = statuses(&news)[0];
    assert!(s.joined());
    assert_eq!(s.ssid.as_deref(), Some(&b"Home"[..]));
    sim.ask(WifiJob::Down);
    assert!(!statuses(&sim.news())[0].joined());
}

#[test]
fn the_desktop_radio_forgets_what_it_is_told_to() {
    let mut sim = SimWifi::default();
    sim.ask(WifiJob::Up);
    sim.ask(WifiJob::Join {
        ssid: "Café Wi-Fi".as_bytes().to_vec(),
        psk: String::new(),
        replace: None,
    });
    let saved = sim
        .news()
        .into_iter()
        .rev()
        .find_map(|n| match n {
            WifiNews::Saved(s) => Some(s),
            _ => None,
        })
        .unwrap();
    assert_eq!(saved.len(), 1, "an open network was not saved");
    sim.ask(WifiJob::Forget(saved[0].id));
    let left = sim.news().into_iter().find_map(|n| match n {
        WifiNews::Saved(s) => Some(s),
        _ => None,
    });
    assert_eq!(left, Some(Vec::new()));
}
