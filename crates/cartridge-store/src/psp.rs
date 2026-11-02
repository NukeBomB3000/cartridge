//! A PSP game's identity, read from its image: the `DISC_ID` of the PARAM.SFO in `PSP_GAME`,
//! which is the serial printed on the UMD (`ULUS-10311`), from a `.iso` or from a `.cso`, whose
//! blocks are deflated apart so that the few sectors that lead to it are all that is read of
//! hundreds of megabytes. A `.pbp` is an eboot that carries its PARAM.SFO at its start, as a
//! PlayStation one does (see [`crate::ps1::serial`]).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::ps1::{serial_of_pbp, sfo_text, tidy_serial};
use crate::Region;

const SECTOR: u64 = 2048;

/// The largest PARAM.SFO and directory worth reading.
const SMALL: u32 = 64 * 1024;

/// The largest compressed block of a `.cso` worth inflating, and the most of it a block makes.
const BLOCK_MAX: u64 = 1 << 20;

/// What a disc image reads its bytes from.
trait Image {
    fn read_at(&mut self, offset: u64, len: usize) -> Option<Vec<u8>>;
}

struct Iso(File);

impl Image for Iso {
    fn read_at(&mut self, offset: u64, len: usize) -> Option<Vec<u8>> {
        let mut out = vec![0u8; len];
        self.0.seek(SeekFrom::Start(offset)).ok()?;
        self.0.read_exact(&mut out).ok()?;
        Some(out)
    }
}

/// A compressed ISO: a header, an index of where each block of the image starts, the blocks.
struct Cso {
    file: File,
    block: u64,
    shift: u32,
    total: u64,
    index: Vec<u32>,
}

impl Cso {
    fn open(mut file: File) -> Option<Cso> {
        let mut head = [0u8; 24];
        file.read_exact(&mut head).ok()?;
        if &head[..4] != b"CISO" {
            return None;
        }
        let total = u64::from_le_bytes(head[8..16].try_into().ok()?);
        let block = u64::from(u32::from_le_bytes(head[16..20].try_into().ok()?));
        let shift = u32::from(head[21]);
        if block == 0 || block > BLOCK_MAX || head[20] > 1 {
            return None;
        }
        let entries = usize::try_from(total.div_ceil(block) + 1).ok()?;
        let mut raw = vec![0u8; entries.checked_mul(4)?];
        file.read_exact(&mut raw).ok()?;
        let index = raw
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        Some(Cso {
            file,
            block,
            shift,
            total,
            index,
        })
    }

    fn block_at(&mut self, n: usize) -> Option<Vec<u8>> {
        let (this, next) = (*self.index.get(n)?, *self.index.get(n + 1)?);
        let start = u64::from(this & 0x7fff_ffff) << self.shift;
        let end = u64::from(next & 0x7fff_ffff) << self.shift;
        let len = usize::try_from(end.checked_sub(start)?).ok()?;
        if len as u64 > BLOCK_MAX * 2 {
            return None;
        }
        let mut raw = vec![0u8; len];
        self.file.seek(SeekFrom::Start(start)).ok()?;
        self.file.read_exact(&mut raw).ok()?;
        if this & 0x8000_0000 != 0 {
            return Some(raw);
        }
        miniz_oxide::inflate::decompress_to_vec(&raw).ok()
    }
}

impl Image for Cso {
    fn read_at(&mut self, offset: u64, len: usize) -> Option<Vec<u8>> {
        if offset.checked_add(len as u64)? > self.total {
            return None;
        }
        let mut out = Vec::with_capacity(len);
        let mut at = offset;
        while out.len() < len {
            let n = usize::try_from(at / self.block).ok()?;
            let block = self.block_at(n)?;
            let from = usize::try_from(at % self.block).ok()?;
            let take = (len - out.len()).min(block.len().checked_sub(from)?);
            if take == 0 {
                return None;
            }
            out.extend_from_slice(&block[from..from + take]);
            at += take as u64;
        }
        Some(out)
    }
}

fn le32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// The extent of the file or folder called `name` in the folder at `dir` (a sector and a size),
/// and whether it is a folder: ISO 9660's records, each led by its own length.
fn find(image: &mut dyn Image, dir: (u32, u32), name: &str) -> Option<(u32, u32, bool)> {
    let bytes = image.read_at(u64::from(dir.0) * SECTOR, dir.1.min(SMALL) as usize)?;
    let mut at = 0;
    while at < bytes.len() {
        let len = *bytes.get(at)? as usize;
        if len == 0 {
            // a record never straddles a sector: the rest of this one is padding
            at = (at / SECTOR as usize + 1) * SECTOR as usize;
            continue;
        }
        let record = bytes.get(at..at + len)?;
        let name_len = *record.get(32)? as usize;
        let raw = record.get(33..33 + name_len)?;
        let full = String::from_utf8_lossy(raw);
        let bare = full.split(';').next().unwrap_or_default();
        if bare.eq_ignore_ascii_case(name) {
            return Some((
                le32(record, 2)?,
                le32(record, 10)?,
                record.get(25)? & 2 != 0,
            ));
        }
        at += len;
    }
    None
}

fn disc_id(image: &mut dyn Image) -> Option<String> {
    let pvd = image.read_at(16 * SECTOR, SECTOR as usize)?;
    if pvd.get(1..6)? != b"CD001" {
        return None;
    }
    let root = (le32(&pvd, 156 + 2)?, le32(&pvd, 156 + 10)?);
    let (game_at, game_len, game_dir) = find(image, root, "PSP_GAME")?;
    let (sfo_at, sfo_len, sfo_dir) = find(image, (game_at, game_len), "PARAM.SFO")?;
    if !game_dir || sfo_dir || sfo_len > SMALL {
        return None;
    }
    let sfo = image.read_at(u64::from(sfo_at) * SECTOR, sfo_len as usize)?;
    sfo_text(&sfo, "DISC_ID").and_then(|id| tidy_serial(&id))
}

/// The serial of the game at `path`: nothing for a file this cannot read, or a format it does
/// not know.
pub fn serial(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "iso" => disc_id(&mut Iso(File::open(path).ok()?)),
        "cso" => disc_id(&mut Cso::open(File::open(path).ok()?)?),
        "pbp" => serial_of_pbp(path),
        _ => None,
    }
}

/// The region a serial's prefix says: the third letter, after the UMD's `UL` or a download's
/// `NP`, is the USA's `U`, Europe's `E`, Japan's `J`, and Korea's `K` or Asia's `A`, which are
/// another region.
pub fn serial_region(serial: &str) -> Option<Region> {
    let p = serial.get(..4)?.to_ascii_uppercase();
    let b = p.as_bytes();
    if !matches!(&p[..2], "UL" | "UC" | "NP") {
        return None;
    }
    match b[2] {
        b'U' => Some(Region::Usa),
        b'E' => Some(Region::Europe),
        b'J' => Some(Region::Japan),
        b'K' | b'A' => Some(Region::Other),
        _ => None,
    }
}

/// Where the game at `path`, named `stem`, comes from: a region in its file name's tags wins
/// over its serial.
pub fn region(path: &Path, stem: &str) -> Option<Region> {
    Region::from_tags(stem).or_else(|| serial(path).as_deref().and_then(serial_region))
}
