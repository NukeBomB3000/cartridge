//! root's password: read from baseos.conf as BaseOS reads it, made up once on a card that sets
//! none, written back the BaseOS way, and given to the running system; and finding the dropbear
//! servers to stop.

use crate::ssh::SHIPPED;

use std::io;
use std::path::{Path, PathBuf};

use cartridge::ssh::{
    conf_password, ensure_password, listeners, listening, new_password, ssh_password,
    with_password, with_root_hash, Password, Proc, SshServer, Trouble,
};

/// A server that keeps the passwords root was given.
#[derive(Default)]
struct Root {
    given: Vec<String>,
    refuses: bool,
}

impl SshServer for Root {
    fn present(&self) -> bool {
        true
    }

    fn start(&mut self) {}

    fn stop(&mut self) {}

    fn baseos_conf(&self, root: &Path) -> Option<PathBuf> {
        Some(root.join("baseos.conf"))
    }

    fn set_root_password(&mut self, password: &str) -> io::Result<()> {
        self.given.push(password.to_string());
        if self.refuses {
            return Err(io::Error::other("refused"));
        }
        Ok(())
    }
}

fn six_digits(password: &str) -> bool {
    password.len() == 6
        && password.bytes().all(|b| b.is_ascii_digit())
        && !password.starts_with('0')
}

/// `conf` with its one `ssh_password=` line, and that password.
fn written(conf: &str) -> (String, String) {
    let password = conf
        .lines()
        .filter_map(|line| line.strip_prefix("ssh_password="))
        .collect::<Vec<_>>();
    assert_eq!(password.len(), 1, "not one password line in {conf:?}");
    (conf.to_string(), password[0].to_string())
}

#[test]
fn the_password_is_read_as_baseos_reads_it() {
    assert_eq!(ssh_password(""), "root", "absent");
    assert_eq!(ssh_password("hostname=rg35xxsp\n"), "root", "absent");
    assert_eq!(ssh_password(SHIPPED), "root", "as BaseOS ships it");
    assert_eq!(
        ssh_password("  #ssh_password=secret\n"),
        "root",
        "commented out after spaces"
    );
    assert_eq!(ssh_password("ssh_password=secret\n"), "secret", "set");
    assert_eq!(
        ssh_password("  ssh_password = secret  \r\n"),
        "secret",
        "spaces round the key and the value"
    );
    assert_eq!(
        ssh_password("ssh_password=two words\n"),
        "two words",
        "a space inside"
    );
    assert_eq!(
        ssh_password("ssh_password=\"quoted\"\n"),
        "\"quoted\"",
        "baseos-config keeps quotes"
    );
    assert_eq!(
        ssh_password("ssh_password=a#b # c\n"),
        "a#b # c",
        "a password keeps its #"
    );
    assert_eq!(ssh_password("ssh_password=x=y\n"), "x=y", "an = inside");
    assert_eq!(ssh_password("ssh_password=\n"), "root", "empty");
    assert_eq!(
        ssh_password("# ssh_password=old\nssh_password=new\n"),
        "new",
        "the line that is not commented out"
    );
    assert_eq!(
        ssh_password("ssh_password=first\nssh_password=second\n"),
        "second",
        "the last line"
    );
    assert_eq!(
        conf_password("ssh_password=first\nssh_password=\n"),
        None,
        "an empty last line leaves root the image's"
    );
}

#[test]
fn a_password_is_six_digits_from_the_random_source() {
    let draws = [0u32, 899_999, 900_000, 123_456_789];
    let bytes: Vec<u8> = draws.iter().flat_map(|n| n.to_le_bytes()).collect();
    let mut random = &bytes[..];
    let made: Vec<String> = (0..draws.len())
        .map(|_| new_password(&mut random).unwrap())
        .collect();
    assert_eq!(made, ["100000", "999999", "100000", "256789"]);
    assert!(made.iter().all(|p| six_digits(p)), "{made:?}");
}

#[test]
fn a_draw_that_would_favour_the_low_numbers_is_drawn_again() {
    let bytes: Vec<u8> = [u32::MAX, u32::MAX - 1, 42]
        .iter()
        .flat_map(|n| n.to_le_bytes())
        .collect();
    assert_eq!(new_password(&mut &bytes[..]).unwrap(), "100042");
    let stuck = [0xff; 4 * 64];
    assert!(new_password(&mut &stuck[..]).is_err(), "a stuck source");
    assert!(new_password(&mut &[1, 2][..]).is_err(), "a source run dry");
}

#[test]
fn passwords_from_dev_urandom_are_six_digits_and_differ() {
    let mut random = std::fs::File::open("/dev/urandom").unwrap();
    let made: Vec<String> = (0..50)
        .map(|_| new_password(&mut random).unwrap())
        .collect();
    assert!(made.iter().all(|p| six_digits(p)), "{made:?}");
    let mut differ = made.clone();
    differ.sort();
    differ.dedup();
    assert!(differ.len() > 40, "{made:?}");
}

#[test]
fn the_commented_line_baseos_ships_becomes_the_password() {
    let out = with_password(SHIPPED.as_bytes(), "482913");
    let want = SHIPPED.replace("# ssh_password=your-password\n", "ssh_password=482913\n");
    assert_eq!(String::from_utf8(out).unwrap(), want);
}

#[test]
fn without_the_commented_line_the_password_goes_under_the_comment_about_it() {
    let conf = SHIPPED
        .replace("# ssh_password=your-password\n", "")
        .replace("# SSH password", "\n# SSH password")
        + "\nhostname=sp\n";
    let out = String::from_utf8(with_password(conf.as_bytes(), "482913")).unwrap();
    let want = conf.replace(
        "BaseOS#settings\n",
        "BaseOS#settings\nssh_password=482913\n",
    );
    assert_eq!(out, want);
}

#[test]
fn without_the_comment_both_go_at_the_end() {
    for (conf, want) in [
        (
            "hostname=sp\n",
            "hostname=sp\n\n# SSH password (plain text).\nssh_password=482913\n",
        ),
        (
            "hostname=sp",
            "hostname=sp\n\n# SSH password (plain text).\nssh_password=482913\n",
        ),
        (
            "hostname=sp\n\n",
            "hostname=sp\n\n# SSH password (plain text).\nssh_password=482913\n",
        ),
        ("", "# SSH password (plain text).\nssh_password=482913\n"),
    ] {
        let out = String::from_utf8(with_password(conf.as_bytes(), "482913")).unwrap();
        assert_eq!(out, want, "{conf:?}");
    }
}

#[test]
fn an_empty_password_line_is_the_one_filled_in() {
    let conf = "# ssh_password=your-password\nssh_password=\nmdns=true\n";
    let out = String::from_utf8(with_password(conf.as_bytes(), "482913")).unwrap();
    assert_eq!(
        out,
        "# ssh_password=your-password\nssh_password=482913\nmdns=true\n"
    );
}

#[test]
fn a_card_written_with_crlf_keeps_its_line_endings_and_its_other_bytes() {
    let conf = b"# caf\xe9\r\n# SSH password (plain text).\r\n# ssh_password=x\r\nmdns=true";
    let out = with_password(conf, "482913");
    assert_eq!(
        out,
        b"# caf\xe9\r\n# SSH password (plain text).\r\nssh_password=482913\r\nmdns=true"
    );
    let conf = b"mdns=true\r\n";
    assert_eq!(
        with_password(conf, "482913"),
        b"mdns=true\r\n\r\n# SSH password (plain text).\r\nssh_password=482913\r\n"
    );
}

#[test]
fn a_card_with_no_password_gets_one_once() {
    let d = tempfile::tempdir().unwrap();
    let conf = d.path().join("baseos.conf");
    std::fs::write(&conf, SHIPPED).unwrap();
    let mut root = Root::default();
    let Password::Made(made) = ensure_password(&conf, &mut root) else {
        panic!("no password made");
    };
    assert!(six_digits(&made), "{made:?}");
    assert_eq!(root.given, [made.as_str()], "root was not given it");
    let (text, kept) = written(&std::fs::read_to_string(&conf).unwrap());
    assert_eq!(kept, made);
    assert_eq!(
        text,
        SHIPPED.replace(
            "# ssh_password=your-password\n",
            &format!("ssh_password={made}\n")
        )
    );

    assert_eq!(ensure_password(&conf, &mut root), Password::Kept, "again");
    assert_eq!(root.given.len(), 1, "root was given another");
    assert_eq!(std::fs::read_to_string(&conf).unwrap(), text, "rewritten");
    let names: Vec<_> = std::fs::read_dir(d.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, ["baseos.conf"], "a temporary file was left behind");
}

#[test]
fn a_password_the_owner_set_is_kept() {
    let d = tempfile::tempdir().unwrap();
    let conf = d.path().join("baseos.conf");
    let owner = SHIPPED.replace("# ssh_password=your-password", "ssh_password=hunter2");
    std::fs::write(&conf, &owner).unwrap();
    let mut root = Root::default();
    assert_eq!(ensure_password(&conf, &mut root), Password::Kept);
    assert!(root.given.is_empty(), "root was given another");
    assert_eq!(std::fs::read_to_string(&conf).unwrap(), owner);
}

#[test]
fn a_card_with_no_baseos_conf_gets_one_with_the_password() {
    let d = tempfile::tempdir().unwrap();
    let conf = d.path().join("baseos.conf");
    let mut root = Root::default();
    let Password::Made(made) = ensure_password(&conf, &mut root) else {
        panic!("no password made");
    };
    assert_eq!(
        std::fs::read_to_string(&conf).unwrap(),
        format!("# SSH password (plain text).\nssh_password={made}\n")
    );
}

#[test]
fn a_card_that_cannot_be_written_gives_root_nothing() {
    let d = tempfile::tempdir().unwrap();
    let conf = d.path().join("gone/baseos.conf");
    let mut root = Root::default();
    assert_eq!(
        ensure_password(&conf, &mut root),
        Password::Failed(Trouble::NotSaved)
    );
    assert!(root.given.is_empty(), "root was given what the card lost");
}

#[test]
fn a_system_that_will_not_take_it_still_has_it_saved_for_the_next_start() {
    let d = tempfile::tempdir().unwrap();
    let conf = d.path().join("baseos.conf");
    std::fs::write(&conf, SHIPPED).unwrap();
    let mut root = Root {
        refuses: true,
        ..Root::default()
    };
    assert_eq!(
        ensure_password(&conf, &mut root),
        Password::Failed(Trouble::NotApplied)
    );
    let (_, kept) = written(&std::fs::read_to_string(&conf).unwrap());
    assert_eq!(root.given, [kept]);
}

#[test]
fn roots_hash_is_replaced_and_nothing_else() {
    let shadow = "root:$6$old:19000:0:99999:7:::\nmessagebus:*:19000:0:99999:7:::\nnobody:*:19000:0:99999:7:::\n";
    assert_eq!(
        with_root_hash(shadow, "$6$salt$new").as_deref(),
        Some("root:$6$salt$new:19000:0:99999:7:::\nmessagebus:*:19000:0:99999:7:::\nnobody:*:19000:0:99999:7:::\n")
    );
    assert_eq!(
        with_root_hash("nobody:*:19000:0:99999:7:::\n", "$6$x"),
        None
    );
    assert_eq!(
        with_root_hash("rooted:*:1:::::\nroot::1:::::", "$6$x").as_deref(),
        Some("rooted:*:1:::::\nroot:$6$x:1:::::\n"),
        "only the line that is root's"
    );
}

/// /proc/net/tcp and tcp6 as the console had them: adb on 127.0.0.1:5037, dropbear on port 22.
const TCP: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:13AD 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 4877 1 0000000000000000 100 0 0 10 0
   1: 00000000:0016 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 2488 1 0000000000000000 100 0 0 10 0
   2: 1401A8C0:0016 0A01A8C0:D431 01 00000000:00000000 02:000A7E6A 00000000     0        0 12885 4 0000000000000000 20 4 29 10 -1
";
const TCP6: &str = "  sl  local_address                         remote_address                        st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000000000000000000000000000:0016 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 2487 1 0000000000000000 100 0 0 10 0
";

#[test]
fn the_sockets_listening_on_the_ssh_port_are_found() {
    assert_eq!(listening(TCP, 22), [2488], "a login made is not listening");
    assert_eq!(listening(TCP6, 22), [2487]);
    assert_eq!(listening(TCP, 5037), [4877]);
    assert!(listening("", 22).is_empty());
}

#[test]
fn only_a_dropbear_listening_is_a_server_to_stop() {
    let proc = |pid, name: &str, sockets: &[u64]| Proc {
        pid,
        name: name.into(),
        sockets: sockets.to_vec(),
    };
    let listening = [2487, 2488];
    let procs = [
        // the server rcS started, and a login it serves, which holds its own connection
        proc(1366, "dropbear", &[2487, 2488, 2489]),
        proc(10660, "dropbear", &[2489, 12885]),
        // something else on the port is not dropbear's to stop
        proc(77, "sshd", &[2488]),
        proc(78, "dropbearkey", &[2487]),
    ];
    assert_eq!(listeners(&procs, &listening), [1366]);
    // its server gone, a login carries on and is not taken for one
    assert!(listeners(&procs[1..2], &listening).is_empty());
}
