use cartridge_store::ps1::{
    boot_serial, disc_label, disc_tag, discs, first_data_track, image_serial, region, serial,
    serial_region, tidy_serial, Track,
};
use cartridge_store::Region;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn a_lone_disc_is_its_own_only_disc() {
    let disc = Path::new("Games/PS1/Vagrant Story (USA).cue");
    assert_eq!(discs(disc), vec![disc.to_path_buf()]);
}

#[test]
fn a_playlist_is_its_discs_in_order_from_its_own_folder() {
    let d = tempdir().unwrap();
    let m3u = d.path().join("Final Fantasy VII (USA).m3u");
    std::fs::write(
        &m3u,
        "\u{feff}# made by hand\nFinal Fantasy VII (USA) (Disc 1).cue\r\n\nFinal Fantasy VII (USA) (Disc 2).cue\nFinal Fantasy VII (USA) (Disc 3).cue\n",
    )
    .unwrap();
    let names: Vec<PathBuf> = discs(&m3u)
        .iter()
        .map(|p| p.strip_prefix(d.path()).unwrap().to_path_buf())
        .collect();
    assert_eq!(
        names,
        vec![
            PathBuf::from("Final Fantasy VII (USA) (Disc 1).cue"),
            PathBuf::from("Final Fantasy VII (USA) (Disc 2).cue"),
            PathBuf::from("Final Fantasy VII (USA) (Disc 3).cue"),
        ]
    );
}

#[test]
fn a_playlist_that_cannot_be_read_has_no_discs() {
    assert!(discs(Path::new("/nowhere/Missing.m3u")).is_empty());
}

#[test]
fn each_disc_keeps_its_label_under_its_own_name() {
    assert_eq!(
        disc_label(
            Path::new("/sd"),
            Path::new("/sd/Games/PS1/Final Fantasy VII (USA) (Disc 2).cue")
        ),
        PathBuf::from("/sd/Labels/PS1/Final Fantasy VII (USA) (Disc 2).png")
    );
}

/// The user data of a tiny ISO 9660 disc, 2048 bytes a sector: the primary volume descriptor at
/// sector 16, the root folder at 18 holding the boot file and SYSTEM.CNF, and SYSTEM.CNF at 19.
pub(crate) fn iso(system_cnf: &str) -> Vec<[u8; 2048]> {
    let mut sectors = vec![[0u8; 2048]; 20];
    let record = |lba: u32, len: u32, flags: u8, name: &[u8]| {
        let mut r = vec![0u8; 33 + name.len() + (name.len() + 1) % 2];
        r[0] = r.len() as u8;
        r[2..6].copy_from_slice(&lba.to_le_bytes());
        r[6..10].copy_from_slice(&lba.to_be_bytes());
        r[10..14].copy_from_slice(&len.to_le_bytes());
        r[14..18].copy_from_slice(&len.to_be_bytes());
        r[25] = flags;
        r[32] = name.len() as u8;
        r[33..33 + name.len()].copy_from_slice(name);
        r
    };
    let pvd = &mut sectors[16];
    pvd[0] = 1;
    pvd[1..6].copy_from_slice(b"CD001");
    pvd[6] = 1;
    let root = record(18, 2048, 2, &[0]);
    pvd[156..156 + root.len()].copy_from_slice(&root);
    sectors[17][0] = 255;
    sectors[17][1..6].copy_from_slice(b"CD001");
    let mut at = 0;
    for r in [
        record(18, 2048, 2, &[0]),
        record(18, 2048, 2, &[1]),
        record(40, 1000, 0, b"SLUS_005.94;1"),
        record(19, system_cnf.len() as u32, 0, b"SYSTEM.CNF;1"),
    ] {
        sectors[18][at..at + r.len()].copy_from_slice(&r);
        at += r.len();
    }
    sectors[19][..system_cnf.len()].copy_from_slice(system_cnf.as_bytes());
    sectors
}

/// The disc as raw 2352-byte sectors: sync, header, for mode 2 the subheader, the data, and the
/// error correction left as zeroes.
pub(crate) fn raw(sectors: &[[u8; 2048]], mode: u8) -> Vec<u8> {
    let mut out = Vec::new();
    for (lba, data) in sectors.iter().enumerate() {
        let mut s = vec![0u8; 2352];
        s[1..11].fill(0xff);
        let at = lba + 150;
        let bcd = |n: usize| (((n / 10) << 4) | (n % 10)) as u8;
        s[12] = bcd(at / 75 / 60);
        s[13] = bcd(at / 75 % 60);
        s[14] = bcd(at % 75);
        s[15] = mode;
        let user = if mode == 1 { 16 } else { 24 };
        if mode == 2 {
            s[18] = 0x08;
            s[22] = 0x08;
        }
        s[user..user + 2048].copy_from_slice(data);
        out.extend(s);
    }
    out
}

fn cooked(sectors: &[[u8; 2048]]) -> Vec<u8> {
    sectors.iter().flatten().copied().collect()
}

pub(crate) const CNF: &str =
    "BOOT = cdrom:\\SLUS_005.94;1\r\nTCB = 4\r\nEVENT = 10\r\nSTACK = 801FFFF0\r\n";

pub(crate) fn cue_for(bin: &str, mode: &str) -> String {
    format!("FILE \"{bin}\" BINARY\r\n  TRACK 01 {mode}\r\n    INDEX 01 00:00:00\r\n")
}

#[test]
fn the_serial_is_read_from_system_cnf_on_a_raw_mode_2_disc() {
    let d = tempdir().unwrap();
    std::fs::write(d.path().join("Game (USA).bin"), raw(&iso(CNF), 2)).unwrap();
    let cue = d.path().join("Game (USA).cue");
    std::fs::write(&cue, cue_for("Game (USA).bin", "MODE2/2352")).unwrap();
    assert_eq!(serial(&cue).as_deref(), Some("SLUS-00594"));
}

#[test]
fn the_serial_is_read_from_a_raw_mode_1_disc_and_a_cooked_one() {
    let d = tempdir().unwrap();
    std::fs::write(d.path().join("m1.bin"), raw(&iso(CNF), 1)).unwrap();
    std::fs::write(d.path().join("m1.cue"), cue_for("m1.bin", "MODE1/2352")).unwrap();
    assert_eq!(
        serial(&d.path().join("m1.cue")).as_deref(),
        Some("SLUS-00594")
    );

    std::fs::write(d.path().join("iso.iso"), cooked(&iso(CNF))).unwrap();
    std::fs::write(d.path().join("iso.cue"), cue_for("iso.iso", "MODE1/2048")).unwrap();
    assert_eq!(
        serial(&d.path().join("iso.cue")).as_deref(),
        Some("SLUS-00594")
    );

    let track = Track {
        start: 0,
        sector: 2048,
    };
    assert_eq!(
        image_serial(&d.path().join("iso.iso"), track).as_deref(),
        Some("SLUS-00594")
    );
}

#[test]
fn the_data_track_is_found_behind_audio_and_a_pregap_in_its_own_file() {
    let d = tempdir().unwrap();
    // the data track starts two seconds into its file, after a pregap of silence
    let mut bin = vec![0u8; 150 * 2352];
    bin.extend(raw(&iso(CNF), 2));
    std::fs::write(d.path().join("Track 2.bin"), bin).unwrap();
    std::fs::write(d.path().join("Track 1.bin"), vec![0u8; 2352 * 4]).unwrap();
    let sheet = "REM a disc that plays music first\r\n\
        FILE \"Track 1.bin\" BINARY\r\n  TRACK 01 AUDIO\r\n    INDEX 01 00:00:00\r\n\
        FILE \"Track 2.bin\" BINARY\r\n  TRACK 02 MODE2/2352\r\n    \
        INDEX 00 00:00:00\r\n    INDEX 01 00:02:00\r\n";
    assert_eq!(
        first_data_track(sheet),
        Some((
            "Track 2.bin".to_string(),
            Track {
                start: 150 * 2352,
                sector: 2352
            }
        ))
    );
    std::fs::write(d.path().join("Game.cue"), sheet).unwrap();
    assert_eq!(
        serial(&d.path().join("Game.cue")).as_deref(),
        Some("SLUS-00594")
    );
}

#[test]
fn a_disc_with_no_serial_to_read_has_none() {
    let d = tempdir().unwrap();
    let psx_exe = raw(&iso("BOOT = cdrom:\\PSX.EXE;1\r\n"), 2);
    std::fs::write(d.path().join("exe.bin"), psx_exe).unwrap();
    std::fs::write(d.path().join("exe.cue"), cue_for("exe.bin", "MODE2/2352")).unwrap();
    assert_eq!(
        serial(&d.path().join("exe.cue")),
        None,
        "PSX.EXE is no serial"
    );

    std::fs::write(d.path().join("blank.bin"), vec![0u8; 2352 * 20]).unwrap();
    std::fs::write(
        d.path().join("blank.cue"),
        cue_for("blank.bin", "MODE2/2352"),
    )
    .unwrap();
    assert_eq!(
        serial(&d.path().join("blank.cue")),
        None,
        "no volume descriptor"
    );

    std::fs::write(d.path().join("short.bin"), vec![0u8; 100]).unwrap();
    std::fs::write(
        d.path().join("short.cue"),
        cue_for("short.bin", "MODE2/2352"),
    )
    .unwrap();
    assert_eq!(
        serial(&d.path().join("short.cue")),
        None,
        "a truncated image"
    );

    std::fs::write(d.path().join("gone.cue"), cue_for("gone.bin", "MODE2/2352")).unwrap();
    assert_eq!(serial(&d.path().join("gone.cue")), None, "a missing image");

    std::fs::write(d.path().join("Game.chd"), b"MComprHD").unwrap();
    assert_eq!(
        serial(&d.path().join("Game.chd")),
        None,
        "a chd is not opened"
    );
}

#[test]
fn a_playlist_has_its_first_discs_serial() {
    let d = tempdir().unwrap();
    std::fs::write(d.path().join("d1.bin"), raw(&iso(CNF), 2)).unwrap();
    std::fs::write(d.path().join("d1.cue"), cue_for("d1.bin", "MODE2/2352")).unwrap();
    let m3u = d.path().join("Game.m3u");
    std::fs::write(&m3u, "d1.cue\nd2.cue\n").unwrap();
    assert_eq!(serial(&m3u).as_deref(), Some("SLUS-00594"));
}

/// A PSP eboot whose PARAM.SFO names its disc `disc_id`.
pub(crate) fn pbp(disc_id: &str) -> Vec<u8> {
    let keys: &[u8] = b"CATEGORY\0DISC_ID\0TITLE\0\0\0";
    let id = [disc_id.as_bytes(), &[0u8; 7]].concat();
    let values: [&[u8]; 3] = [b"ME\0\0", &id, b"Game\0\0\0\0"];
    let key_offsets = [0u16, 9, 17];
    let key_table = 20 + 16 * values.len() as u32;
    let data_table = key_table + keys.len() as u32;
    let mut sfo = Vec::new();
    sfo.extend(b"\0PSF");
    sfo.extend(0x0101u32.to_le_bytes());
    sfo.extend(key_table.to_le_bytes());
    sfo.extend(data_table.to_le_bytes());
    sfo.extend((values.len() as u32).to_le_bytes());
    let mut data_at = 0u32;
    for (v, key_at) in values.iter().zip(key_offsets) {
        sfo.extend(key_at.to_le_bytes());
        sfo.extend(0x0204u16.to_le_bytes());
        sfo.extend((v.len() as u32).to_le_bytes());
        sfo.extend((v.len() as u32).to_le_bytes());
        sfo.extend(data_at.to_le_bytes());
        data_at += v.len() as u32;
    }
    sfo.extend(keys);
    for v in values {
        sfo.extend(v);
    }
    let sfo_at = 0x28u32;
    let next = sfo_at + sfo.len() as u32;
    let mut out = Vec::new();
    out.extend(b"\0PBP");
    out.extend(0x10000u32.to_le_bytes());
    out.extend(sfo_at.to_le_bytes());
    for _ in 0..7 {
        out.extend(next.to_le_bytes());
    }
    out.extend(&sfo);
    out.extend([0u8; 64]);
    out
}

#[test]
fn a_pbp_is_named_by_its_disc_id() {
    let d = tempdir().unwrap();
    let eboot = d.path().join("Game (Europe).pbp");
    std::fs::write(&eboot, pbp("SCES00344")).unwrap();
    assert_eq!(serial(&eboot).as_deref(), Some("SCES-00344"));
    std::fs::write(&eboot, b"not an eboot at all").unwrap();
    assert_eq!(serial(&eboot), None);
}

#[test]
fn a_serial_is_tidied_the_way_it_is_printed() {
    assert_eq!(tidy_serial("SLUS_005.94").as_deref(), Some("SLUS-00594"));
    assert_eq!(tidy_serial("slps00700").as_deref(), Some("SLPS-00700"));
    assert_eq!(tidy_serial("SCES-00344").as_deref(), Some("SCES-00344"));
    assert_eq!(tidy_serial("PSX.EXE"), None);
    assert_eq!(tidy_serial("SLUS_0059"), None);
    assert_eq!(
        boot_serial("BOOT=cdrom:SCPS_100.01;1\nTCB=4\n").as_deref(),
        Some("SCPS-10001"),
        "no spaces and no backslash"
    );
    assert_eq!(boot_serial("TCB = 4\n"), None);
}

#[test]
fn a_serials_prefix_says_where_the_disc_comes_from() {
    for (serial, region) in [
        ("SLUS-00594", Region::Usa),
        ("SCUS-94163", Region::Usa),
        ("SLES-02080", Region::Europe),
        ("SCES-00344", Region::Europe),
        ("SLPS-00700", Region::Japan),
        ("SCPS-10001", Region::Japan),
        ("SLPM-86114", Region::Japan),
        ("SIPS-60015", Region::Japan),
        ("SCPM-85000", Region::Japan),
        ("SCKA-20001", Region::Other),
    ] {
        assert_eq!(serial_region(serial), Some(region), "{serial}");
    }
    assert_eq!(serial_region("ABCD-12345"), None);
    assert_eq!(serial_region(""), None);
}

#[test]
fn a_tag_in_the_name_wins_over_the_serial() {
    let d = tempdir().unwrap();
    std::fs::write(d.path().join("g.bin"), raw(&iso(CNF), 2)).unwrap();
    let cue = d.path().join("Game (Europe).cue");
    std::fs::write(&cue, cue_for("g.bin", "MODE2/2352")).unwrap();
    assert_eq!(region(&cue, "Game (Europe)"), Some(Region::Europe));
    assert_eq!(region(&cue, "Game"), Some(Region::Usa), "the SLUS serial");
    assert_eq!(region(Path::new("/nowhere/Game.chd"), "Game"), None);
}

#[test]
fn a_disc_tag_is_read_off_a_name() {
    assert_eq!(
        disc_tag("Final Fantasy VII (USA) (Disc 2)"),
        Some(("Final Fantasy VII (USA)".to_string(), 2))
    );
    assert_eq!(
        disc_tag("Metal Gear Solid (Disc 1 of 2) (Europe)"),
        Some(("Metal Gear Solid (Europe)".to_string(), 1))
    );
    assert_eq!(
        disc_tag("Parasite Eve (disc 10)"),
        Some(("Parasite Eve".to_string(), 10))
    );
    assert_eq!(disc_tag("Vagrant Story (USA)"), None);
    assert_eq!(disc_tag("Discworld (USA)"), None);
    assert_eq!(disc_tag("(Disc 1)"), None, "a name that is only the tag");
}
