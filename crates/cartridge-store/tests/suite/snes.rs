use cartridge_store::snes::{header, region, Header};
use cartridge_store::Region;

const LOROM: usize = 0x7fc0;
const HIROM: usize = 0xffc0;
const EXHIROM: usize = 0x40_ffc0;

/// A ROM `len` bytes long, filled with `fill`, with a header at `at` naming `title`, mapped by
/// `map`, for `destination`: its checksum and complement adding up, its reset vector into ROM.
fn rom(len: usize, fill: u8, at: usize, title: &[u8], map: u8, destination: u8) -> Vec<u8> {
    let mut rom = vec![fill; len];
    put_header(&mut rom, at, title, map, destination);
    rom
}

fn put_header(rom: &mut [u8], at: usize, title: &[u8], map: u8, destination: u8) {
    let h = &mut rom[at..at + 0x40];
    h[..21].fill(b' ');
    h[..title.len()].copy_from_slice(title);
    h[0x15] = map;
    h[0x16] = 0x02;
    h[0x17] = 0x0a;
    h[0x18] = 0x03;
    h[0x19] = destination;
    h[0x1a] = 0x01;
    h[0x1b] = 0x00;
    let sum: u16 = 0x5a3c;
    h[0x1c..0x1e].copy_from_slice(&(!sum).to_le_bytes());
    h[0x1e..0x20].copy_from_slice(&sum.to_le_bytes());
    h[0x3c..0x3e].copy_from_slice(&0x8000u16.to_le_bytes());
}

/// `rom` with the 512 bytes a copier put in front of it.
fn copied(rom: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 512];
    out.extend_from_slice(rom);
    out
}

fn read(rom: &[u8]) -> Option<Header> {
    Header::parse(rom)
}

#[test]
fn a_lorom_header_reads_its_title_and_where_it_was_sold() {
    let got = read(&rom(0x8_0000, 0xff, LOROM, b"SUPER METROID", 0x30, 0x01)).unwrap();
    assert_eq!(got.title, "SUPER METROID");
    assert_eq!(got.region, Some(Region::Usa));
}

#[test]
fn a_hirom_header_is_found_past_a_first_bank_of_code() {
    // the first bank is a game's bytes, which happen to hold a plausible title where a LoROM
    // header would be, but nothing else of one
    let mut bytes = rom(0x10_0000, 0x4c, HIROM, b"SEIKEN DENSETSU 3", 0x31, 0x00);
    bytes[LOROM..LOROM + 21].copy_from_slice(b"JUST SOME TEXT IN ROM");
    let got = read(&bytes).unwrap();
    assert_eq!(got.title, "SEIKEN DENSETSU 3");
    assert_eq!(got.region, Some(Region::Japan));
}

#[test]
fn of_two_headers_the_one_whose_checksum_adds_up_wins() {
    let mut bytes = rom(0x10_0000, 0x00, HIROM, b"THE REAL ONE", 0x21, 0x02);
    put_header(&mut bytes, LOROM, b"A DECOY", 0x20, 0x01);
    // the decoy's checksum no longer adds up to its complement
    bytes[LOROM + 0x1e] ^= 0x10;
    let got = read(&bytes).unwrap();
    assert_eq!(got.title, "THE REAL ONE");
    assert_eq!(got.region, Some(Region::Europe));
}

#[test]
fn a_copier_header_moves_the_header_512_bytes_on() {
    let plain = rom(0x8_0000, 0xff, LOROM, b"ZELDA", 0x20, 0x09);
    let got = read(&copied(&plain)).unwrap();
    assert_eq!(got.title, "ZELDA");
    assert_eq!(got.region, Some(Region::Europe), "Germany is in Europe");
}

#[test]
fn an_exhirom_header_is_found_past_four_megabytes() {
    let bytes = rom(0x50_0000, 0x00, EXHIROM, b"TALES OF PHANTASIA", 0x35, 0x00);
    assert_eq!(read(&bytes).unwrap().title, "TALES OF PHANTASIA");
}

#[test]
fn each_destination_code_names_its_region() {
    for (code, want) in [
        (0x00, Region::Japan),
        (0x01, Region::Usa),
        (0x0f, Region::Usa),
        (0x02, Region::Europe),
        (0x06, Region::Europe),
        (0x0c, Region::Europe),
        (0x11, Region::Europe),
        (0x0d, Region::Other),
        (0x10, Region::Other),
        (0x14, Region::Other),
    ] {
        let got = read(&rom(0x8_0000, 0xff, LOROM, b"GAME", 0x20, code)).unwrap();
        assert_eq!(got.region, Some(want), "destination {code:#04x}");
    }
}

#[test]
fn a_title_in_katakana_reads_as_katakana() {
    // スーパーマリオ in JIS X 0201
    let title = [0xbd, 0xb0, 0xca, 0xdf, 0xb0, 0xcf, 0xd8, 0xb5];
    let got = read(&rom(0x8_0000, 0xff, LOROM, &title, 0x20, 0x00)).unwrap();
    assert_eq!(got.title, "ｽｰﾊﾟｰﾏﾘｵ");
}

#[test]
fn a_rom_with_nothing_like_a_header_has_none() {
    assert_eq!(read(&vec![0u8; 0x8_0000]), None, "blank");
    assert_eq!(read(&vec![0xffu8; 0x8_0000]), None, "erased");
    assert_eq!(read(&[0u8; 0x100]), None, "too short to hold one");
    let noise: Vec<u8> = (0..0x8_0000u32)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
        .collect();
    assert_eq!(read(&noise), None, "noise");
}

#[test]
fn a_file_name_tag_wins_over_the_header() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("game.sfc");
    std::fs::write(&path, rom(0x8_0000, 0xff, LOROM, b"GAME", 0x20, 0x01)).unwrap();
    assert_eq!(header(&path).unwrap().title, "GAME");
    assert_eq!(region(&path, "Game"), Some(Region::Usa), "the header");
    assert_eq!(
        region(&path, "Game (Europe)"),
        Some(Region::Europe),
        "the tag"
    );
    assert_eq!(region(&path, "Game (Japan)"), Some(Region::Japan));
}

#[test]
fn a_file_that_is_not_there_or_has_no_header_has_no_region_but_its_tags() {
    let d = tempfile::tempdir().unwrap();
    let blank = d.path().join("blank.sfc");
    std::fs::write(&blank, vec![0u8; 0x8_0000]).unwrap();
    assert_eq!(header(&blank), None);
    assert_eq!(region(&blank, "Blank"), None);
    let missing = d.path().join("missing.sfc");
    assert_eq!(region(&missing, "Missing"), None);
    assert_eq!(region(&missing, "Missing (USA)"), Some(Region::Usa));
}

#[test]
fn the_file_reads_as_its_bytes_do_copier_header_and_all() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("copied.smc");
    let bytes = copied(&rom(0x10_0000, 0x00, HIROM, b"CHRONO TRIGGER", 0x31, 0x01));
    std::fs::write(&path, &bytes).unwrap();
    assert_eq!(header(&path), read(&bytes));
    assert_eq!(header(&path).unwrap().title, "CHRONO TRIGGER");
}
