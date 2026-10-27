//! A SNES ROM's internal header: the 64 bytes at the end of the first bank the console reads
//! its vectors from. Where that is depends on how the cart maps its ROM, which the file does not
//! say, so each place it could be is read and the one that looks most like a header wins.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::Region;

/// What a SNES ROM's header says about it.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Header {
    pub title: String,
    pub region: Option<Region>,
}

/// Where the header can be in a ROM with no copier header: at the end of the first 32 KiB bank
/// of a LoROM cart, of the first 64 KiB bank of a HiROM one, and of the first bank past 4 MiB of
/// an ExHiROM one.
const LOROM: u64 = 0x7fc0;
const HIROM: u64 = 0xffc0;
const EXHIROM: u64 = 0x40_ffc0;

/// How the ROM is mapped, the low nibble of the map mode byte, each place's own.
const LOROM_MAPS: &[u8] = &[0x0, 0x2, 0x3];
const HIROM_MAPS: &[u8] = &[0x1, 0xa];
const EXHIROM_MAPS: &[u8] = &[0x5];

/// A copier such as the Super Magicom put 512 bytes of its own in front of the ROM, which
/// leaves the file 512 bytes past a whole number of KiB.
const COPIER: u64 = 512;

/// The header's fields, from its first byte.
const TITLE_LEN: usize = 21;
const MAP: usize = 0x15;
const ROM_SIZE: usize = 0x17;
const DESTINATION: usize = 0x19;
const COMPLEMENT: usize = 0x1c;
const CHECKSUM: usize = 0x1e;
const RESET: usize = 0x3c;
const LEN: usize = 0x40;

/// A header scoring less than this is taken for the bytes of a game, not a header.
const MIN_SCORE: u32 = 4;

impl Header {
    /// The header of the ROM in `rom`, the whole file as it is on the card.
    pub fn parse(rom: &[u8]) -> Option<Header> {
        best(rom.len() as u64, |at| {
            let at = usize::try_from(at).ok()?;
            rom.get(at..at.checked_add(LEN)?)?.try_into().ok()
        })
    }
}

/// The header of the ROM at `path`, if it has one that reads.
pub fn header(path: &Path) -> Option<Header> {
    let mut f = File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    best(len, |at| {
        let mut block = [0u8; LEN];
        f.seek(SeekFrom::Start(at)).ok()?;
        f.read_exact(&mut block).ok()?;
        Some(block)
    })
}

/// Where the game at `path`, named `stem`, comes from: a region in its file name's tags wins
/// over its header.
pub fn region(path: &Path, stem: &str) -> Option<Region> {
    Region::from_tags(stem).or_else(|| header(path).and_then(|h| h.region))
}

/// The header of a ROM file `len` bytes long that reads its 64-byte blocks with `read`: of
/// every place one could be, the one that scores best, if any scores well enough.
fn best(len: u64, mut read: impl FnMut(u64) -> Option<[u8; LEN]>) -> Option<Header> {
    let skip = if len % 1024 == COPIER { COPIER } else { 0 };
    [
        (LOROM, LOROM_MAPS),
        (HIROM, HIROM_MAPS),
        (EXHIROM, EXHIROM_MAPS),
    ]
    .into_iter()
    .filter_map(|(at, maps)| {
        let block = read(at + skip)?;
        Some((score(&block, maps), block))
    })
    .filter(|(score, _)| *score >= MIN_SCORE)
    // the first of two that score the same: LoROM before HiROM, as the console's own order
    .reduce(|a, b| if b.0 > a.0 { b } else { a })
    .map(|(_, block)| Header {
        title: title(&block[..TITLE_LEN]),
        region: Some(destination(block[DESTINATION])),
    })
}

/// How much `block` looks like a header for a cart mapped one of `maps` ways: its checksum
/// and its complement adding up counts most, then a map mode that belongs where it was found,
/// a title in letters, a reset vector into ROM, and a ROM size and destination in range.
fn score(block: &[u8; LEN], maps: &[u8]) -> u32 {
    let word = |at: usize| u16::from_le_bytes([block[at], block[at + 1]]);
    let mut score = 0;
    if word(CHECKSUM).wrapping_add(word(COMPLEMENT)) == 0xffff {
        score += 4;
    }
    let map = block[MAP];
    if matches!(map & 0xe0, 0x20) {
        score += 1;
        if maps.contains(&(map & 0x0f)) {
            score += 2;
        }
    }
    if block[..TITLE_LEN].iter().all(|&b| printable(b)) {
        score += 2;
    }
    if word(RESET) >= 0x8000 {
        score += 1;
    }
    if (0x07..=0x0d).contains(&block[ROM_SIZE]) {
        score += 1;
    }
    if block[DESTINATION] <= 0x14 {
        score += 1;
    }
    score
}

/// Whether `b` can be in a title: plain ASCII, or JIS X 0201's half-width katakana.
fn printable(b: u8) -> bool {
    (0x20..0x7f).contains(&b) || (0xa1..=0xdf).contains(&b)
}

/// The title's bytes as text, the katakana as their half-width Unicode forms, without the
/// spaces it is padded with.
fn title(bytes: &[u8]) -> String {
    let text: String = bytes
        .iter()
        .map(|&b| match b {
            0x20..=0x7e => b as char,
            0xa1..=0xdf => char::from_u32(0xff61 + u32::from(b - 0xa1)).unwrap_or(' '),
            _ => ' ',
        })
        .collect();
    text.trim().to_string()
}

/// The region a destination code names: Japan; the USA, and Canada, which had the same carts;
/// Europe for the PAL codes after them, and Australia; anything else is other.
fn destination(code: u8) -> Region {
    match code {
        0x00 => Region::Japan,
        0x01 | 0x0f => Region::Usa,
        0x02..=0x0c | 0x11 => Region::Europe,
        _ => Region::Other,
    }
}
