use cartridge_store::psp::{region, serial, serial_region};
use cartridge_store::Region;

const SECTOR: usize = 2048;

fn put_le32(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// A directory record for `name`, the way ISO 9660 writes one.
fn record(name: &str, sector: u32, size: u32, dir: bool) -> Vec<u8> {
    let mut r = vec![0u8; 33 + name.len() + (name.len() + 1) % 2];
    r[0] = r.len() as u8;
    put_le32(&mut r, 2, sector);
    put_le32(&mut r, 10, size);
    r[25] = if dir { 2 } else { 0 };
    r[32] = name.len() as u8;
    r[33..33 + name.len()].copy_from_slice(name.as_bytes());
    r
}

/// A PARAM.SFO that says `disc_id`.
fn sfo(disc_id: &str) -> Vec<u8> {
    let key = b"DISC_ID\0";
    let value = {
        let mut v = disc_id.as_bytes().to_vec();
        v.push(0);
        v
    };
    let mut out = vec![0u8; 20 + 16];
    out[..4].copy_from_slice(b"\0PSF");
    put_le32(&mut out, 4, 0x0101);
    put_le32(&mut out, 8, 36); // the keys
    put_le32(&mut out, 12, 36 + key.len() as u32); // the data
    put_le32(&mut out, 16, 1);
    out[20..22].copy_from_slice(&0u16.to_le_bytes());
    out[22..24].copy_from_slice(&0x0204u16.to_le_bytes());
    put_le32(&mut out, 24, value.len() as u32);
    put_le32(&mut out, 28, value.len() as u32);
    put_le32(&mut out, 32, 0);
    out.extend_from_slice(key);
    out.extend_from_slice(&value);
    out
}

/// An ISO 9660 image with `PSP_GAME/PARAM.SFO` for `disc_id`, padded out so that it is longer
/// than the few sectors that matter.
fn iso(disc_id: &str) -> Vec<u8> {
    let sfo = sfo(disc_id);
    let mut img = vec![0u8; 40 * SECTOR];
    let pvd = 16 * SECTOR;
    img[pvd] = 1;
    img[pvd + 1..pvd + 6].copy_from_slice(b"CD001");
    let root = record("", 20, SECTOR as u32, true);
    img[pvd + 156..pvd + 156 + root.len()].copy_from_slice(&root);
    let game = record("PSP_GAME", 21, SECTOR as u32, true);
    img[20 * SECTOR..20 * SECTOR + game.len()].copy_from_slice(&game);
    let file = record("PARAM.SFO;1", 22, sfo.len() as u32, false);
    img[21 * SECTOR..21 * SECTOR + file.len()].copy_from_slice(&file);
    img[22 * SECTOR..22 * SECTOR + sfo.len()].copy_from_slice(&sfo);
    img
}

/// The same image as a CSO: blocks of a sector, each deflated or, alternately, kept as they are.
fn cso(image: &[u8]) -> Vec<u8> {
    let blocks = image.len().div_ceil(SECTOR);
    let mut out = vec![0u8; 24 + (blocks + 1) * 4];
    out[..4].copy_from_slice(b"CISO");
    put_le32(&mut out, 4, 24);
    out[8..16].copy_from_slice(&(image.len() as u64).to_le_bytes());
    put_le32(&mut out, 16, SECTOR as u32);
    out[20] = 1;
    out[21] = 0;
    for (i, block) in image.chunks(SECTOR).enumerate() {
        let at = out.len() as u32;
        if i % 2 == 0 {
            out.extend_from_slice(&miniz_oxide::deflate::compress_to_vec(block, 6));
            put_le32(&mut out, 24 + i * 4, at);
        } else {
            out.extend_from_slice(block);
            put_le32(&mut out, 24 + i * 4, at | 0x8000_0000);
        }
    }
    let end = out.len() as u32;
    put_le32(&mut out, 24 + blocks * 4, end);
    out
}

fn write(name: &str, bytes: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join(name);
    std::fs::write(&p, bytes).unwrap();
    (d, p)
}

#[test]
fn an_iso_says_the_serial_printed_on_its_umd() {
    let (_d, p) = write("Racing Game.iso", &iso("ULUS10311"));
    assert_eq!(serial(&p).as_deref(), Some("ULUS-10311"));
}

#[test]
fn a_cso_says_the_same_without_being_read_whole() {
    let (_d, p) = write("Racing Game.cso", &cso(&iso("ULES01234")));
    assert_eq!(serial(&p).as_deref(), Some("ULES-01234"));
}

#[test]
fn a_file_that_is_no_disc_has_no_serial() {
    let (_d, p) = write("Nothing.iso", &vec![7u8; 100 * 1024]);
    assert_eq!(serial(&p), None);
    let (_d, p) = write("Nothing.cso", b"CISO but not much of one");
    assert_eq!(serial(&p), None);
    let (_d, p) = write("Nothing.txt", &iso("ULUS10311"));
    assert_eq!(serial(&p), None);
}

#[test]
fn a_serials_prefix_says_where_the_game_comes_from() {
    assert_eq!(serial_region("ULUS-10311"), Some(Region::Usa));
    assert_eq!(serial_region("UCUS-98632"), Some(Region::Usa));
    assert_eq!(serial_region("ULES-01234"), Some(Region::Europe));
    assert_eq!(serial_region("NPEZ-00001"), Some(Region::Europe));
    assert_eq!(serial_region("ULJM-05123"), Some(Region::Japan));
    assert_eq!(serial_region("ULKS-46004"), Some(Region::Other));
    assert_eq!(
        serial_region("SLUS-00594"),
        None,
        "a PlayStation disc's serial"
    );
}

#[test]
fn a_region_in_the_file_name_wins_over_the_serial() {
    let (_d, p) = write("Racing Game (Europe).iso", &iso("ULUS10311"));
    assert_eq!(region(&p, "Racing Game (Europe)"), Some(Region::Europe));
    assert_eq!(region(&p, "Racing Game"), Some(Region::Usa));
}
