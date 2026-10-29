use std::path::Path;

pub fn cover(path: &Path, w: u32, h: u32) -> Option<Vec<u8>> {
    let (src, sw, sh) = decode_for(path, (w, h))?;
    if sw == 0 || sh == 0 {
        return None;
    }

    let scale = (w as f32 / sw as f32).max(h as f32 / sh as f32);
    let ox = (sw as f32 - w as f32 / scale) / 2.0;
    let oy = (sh as f32 - h as f32 / scale) / 2.0;

    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        let y0 = oy + y as f32 / scale;
        let y1 = oy + (y + 1) as f32 / scale;
        for x in 0..w {
            let x0 = ox + x as f32 / scale;
            let x1 = ox + (x + 1) as f32 / scale;
            let px = box_average(&src, sw, sh, x0, y0, x1, y1);
            out[((y * w + x) * 4) as usize..][..4].copy_from_slice(&px);
        }
    }
    Some(out)
}

/// The png at `path` as RGBA, only if it is exactly `w` by `h`.
pub fn exact(path: &Path, w: u32, h: u32) -> Option<Vec<u8>> {
    let (rgba, sw, sh) = decode(path)?;
    ((sw, sh) == (w, h)).then_some(rgba)
}

/// How big the png at `path` is, read from its header alone.
pub fn size(path: &Path) -> Option<(u32, u32)> {
    let file = std::fs::File::open(path).ok()?;
    let reader = png::Decoder::new(std::io::BufReader::new(file))
        .read_info()
        .ok()?;
    Some((reader.info().width, reader.info().height))
}

/// `rgba`, `w` by `h`, as a png: without its alpha when every pixel is opaque, as a cart's label
/// is, and each row filtered as suits it. Compressing harder saves under one byte in a hundred
/// on a label and takes a third as long again.
pub fn encode(rgba: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
    let opaque = rgba.chunks_exact(4).all(|px| px[3] == 255);
    let rgb: Vec<u8>;
    let (colour, data) = if opaque {
        rgb = rgba
            .chunks_exact(4)
            .flat_map(|px| &px[..3])
            .copied()
            .collect();
        (png::ColorType::Rgb, &rgb[..])
    } else {
        (png::ColorType::Rgba, rgba)
    };
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(colour);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_adaptive_filter(png::AdaptiveFilterType::Adaptive);
    let mut writer = enc.write_header().ok()?;
    writer.write_image_data(data).ok()?;
    writer.finish().ok()?;
    Some(out)
}

fn box_average(src: &[u8], sw: u32, sh: u32, x0: f32, y0: f32, x1: f32, y1: f32) -> [u8; 4] {
    let xa = (x0.floor().max(0.0) as u32).min(sw - 1);
    let ya = (y0.floor().max(0.0) as u32).min(sh - 1);
    let xb = ((x1.ceil() as u32).max(xa + 1)).min(sw);
    let yb = ((y1.ceil() as u32).max(ya + 1)).min(sh);

    let mut acc = [0u32; 4];
    let mut n = 0u32;
    for y in ya..yb {
        for x in xa..xb {
            let i = ((y * sw + x) * 4) as usize;
            for c in 0..4 {
                acc[c] += src[i + c] as u32;
            }
            n += 1;
        }
    }
    let mut out = [0u8; 4];
    for c in 0..4 {
        out[c] = ((acc[c] + n / 2) / n) as u8;
    }
    out
}

/// The picture at `path`, a png or a jpeg, as RGBA with its width and height, a jpeg decoded
/// no bigger than it needs to be to cover `want`: a photo of 24 megapixels is read at an eighth
/// of its size, which is all a label or a screen of this size can show of it.
pub fn decode_for(path: &Path, want: (u32, u32)) -> Option<(Vec<u8>, u32, u32)> {
    if is_jpeg(path) {
        decode_jpeg(path, want)
    } else {
        decode(path)
    }
}

fn is_jpeg(path: &Path) -> bool {
    use std::io::Read;
    let mut magic = [0u8; 2];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut magic))
        .is_ok()
        && magic == [0xff, 0xd8]
}

/// The orientation (1 to 8) a jpeg's Exif data says it is to be seen in.
pub(crate) fn exif_orientation(exif: &[u8]) -> Option<u8> {
    let tiff = exif.strip_prefix(b"Exif\0\0").unwrap_or(exif);
    let little = match tiff.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16_at = |at: usize| -> Option<u16> {
        let b = tiff.get(at..at + 2)?;
        Some(if little {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        })
    };
    let u32_at = |at: usize| -> Option<u32> {
        let b = tiff.get(at..at + 4)?;
        Some(if little {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        })
    };
    if u16_at(2)? != 42 {
        return None;
    }
    let ifd = u32_at(4)? as usize;
    let count = u16_at(ifd)? as usize;
    (0..count)
        .find_map(|i| {
            let entry = ifd + 2 + i * 12;
            (u16_at(entry)? == 0x0112)
                .then(|| u16_at(entry + 8))
                .flatten()
        })
        .map(|v| v as u8)
        .filter(|v| (1..=8).contains(v))
}

/// `rgba`, `w` by `h`, turned to be seen upright as Exif orientation `o` says, and its new size.
pub(crate) fn orient(rgba: Vec<u8>, w: u32, h: u32, o: u8) -> (Vec<u8>, u32, u32) {
    if o == 1 {
        return (rgba, w, h);
    }
    let (nw, nh) = if o >= 5 { (h, w) } else { (w, h) };
    let mut out = vec![0u8; rgba.len()];
    for y in 0..h {
        for x in 0..w {
            // where the pixel at (x, y) lands, for each of the orientations
            let (dx, dy) = match o {
                2 => (w - 1 - x, y),
                3 => (w - 1 - x, h - 1 - y),
                4 => (x, h - 1 - y),
                5 => (y, x),
                6 => (h - 1 - y, x),
                7 => (h - 1 - y, w - 1 - x),
                _ => (y, w - 1 - x),
            };
            let from = ((y * w + x) * 4) as usize;
            let to = ((dy * nw + dx) * 4) as usize;
            out[to..to + 4].copy_from_slice(&rgba[from..from + 4]);
        }
    }
    (out, nw, nh)
}

fn decode_jpeg(path: &Path, want: (u32, u32)) -> Option<(Vec<u8>, u32, u32)> {
    let file = std::fs::File::open(path).ok()?;
    let mut dec = jpeg_decoder::Decoder::new(std::io::BufReader::new(file));
    dec.read_info().ok()?;
    // a quarter turned picture is wanted the other way round
    let orientation = dec.exif_data().and_then(exif_orientation).unwrap_or(1);
    let (rw, rh) = if orientation >= 5 {
        (want.1, want.0)
    } else {
        want
    };
    dec.scale(rw.clamp(1, 65535) as u16, rh.clamp(1, 65535) as u16)
        .ok()?;
    let pixels = dec.decode().ok()?;
    let info = dec.info()?;
    let (w, h) = (u32::from(info.width), u32::from(info.height));
    let n = (w * h) as usize;
    let mut rgba = vec![255u8; n * 4];
    match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => {
            for i in 0..n {
                rgba[i * 4..i * 4 + 3].copy_from_slice(pixels.get(i * 3..i * 3 + 3)?);
            }
        }
        jpeg_decoder::PixelFormat::L8 => {
            for i in 0..n {
                rgba[i * 4..i * 4 + 3].fill(*pixels.get(i)?);
            }
        }
        _ => return None,
    }
    Some(orient(rgba, w, h, orientation))
}

/// The png at `path` as RGBA, with its width and height.
pub(crate) fn decode(path: &Path) -> Option<(Vec<u8>, u32, u32)> {
    let file = std::fs::File::open(path).ok()?;
    let mut dec = png::Decoder::new(std::io::BufReader::new(file));
    dec.set_transformations(png::Transformations::normalize_to_color8());
    dec.set_limits(png::Limits { bytes: 64 << 20 });

    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let src = &buf[..info.buffer_size()];
    let n = (info.width * info.height) as usize;

    let mut rgba = vec![255u8; n * 4];
    match info.color_type {
        png::ColorType::Rgba => rgba.copy_from_slice(src),
        png::ColorType::Rgb => {
            for i in 0..n {
                rgba[i * 4..i * 4 + 3].copy_from_slice(&src[i * 3..i * 3 + 3]);
            }
        }
        png::ColorType::Grayscale => {
            for i in 0..n {
                rgba[i * 4..i * 4 + 3].fill(src[i]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for i in 0..n {
                rgba[i * 4..i * 4 + 3].fill(src[i * 2]);
                rgba[i * 4 + 3] = src[i * 2 + 1];
            }
        }
        png::ColorType::Indexed => return None,
    }
    Some((rgba, info.width, info.height))
}

#[cfg(test)]
mod jpeg_tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    fn px(rgba: &[u8], w: u32, x: u32, y: u32) -> [u8; 3] {
        let i = ((y * w + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    }

    fn is_red(c: [u8; 3]) -> bool {
        c[0] > 200 && c[2] < 60
    }

    fn is_blue(c: [u8; 3]) -> bool {
        c[2] > 200 && c[0] < 60
    }

    #[test]
    fn a_jpeg_is_decoded_to_rgba() {
        let (rgba, w, h) = decode_for(&fixture("two-halves.jpg"), (32, 16)).expect("a jpeg");
        assert_eq!((w, h), (32, 16));
        assert!(is_red(px(&rgba, w, 2, 8)));
        assert!(is_blue(px(&rgba, w, 29, 8)));
    }

    #[test]
    fn a_jpeg_marked_turned_is_decoded_upright() {
        // 32 by 16, red left and blue right, marked as turned a quarter clockwise to be seen:
        // it comes out 16 by 32 with red on top
        let (rgba, w, h) = decode_for(&fixture("two-halves-rot6.jpg"), (16, 32)).expect("a jpeg");
        assert_eq!((w, h), (16, 32));
        assert!(is_red(px(&rgba, w, 8, 2)), "the left half is on top");
        assert!(is_blue(px(&rgba, w, 8, 29)), "the right half is under it");
    }

    #[test]
    fn the_orientation_is_read_from_the_exif_data_either_way_round() {
        // big and little endian TIFF headers, one entry, orientation 6
        let little = [
            b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0,
        ];
        let big = [
            b'M', b'M', 0, 42, 0, 0, 0, 8, 0, 1, 0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, 6, 0, 0, 0, 0,
        ];
        assert_eq!(exif_orientation(&little), Some(6));
        assert_eq!(exif_orientation(&big), Some(6));
        let mut with_header = b"Exif\0\0".to_vec();
        with_header.extend_from_slice(&little);
        assert_eq!(exif_orientation(&with_header), Some(6));
        assert_eq!(exif_orientation(b"junk"), None);
    }

    #[test]
    fn every_orientation_lands_the_corner_where_the_exif_data_says() {
        // a 2 by 1 picture, `a` then `b`
        let src = vec![1, 1, 1, 255, 2, 2, 2, 255];
        let at = |o: u8| {
            let (out, w, h) = orient(src.clone(), 2, 1, o);
            let first: Vec<u8> = out.chunks_exact(4).map(|p| p[0]).collect();
            (first, w, h)
        };
        assert_eq!(at(1), (vec![1, 2], 2, 1));
        assert_eq!(at(2), (vec![2, 1], 2, 1), "mirrored");
        assert_eq!(at(3), (vec![2, 1], 2, 1), "turned half way");
        assert_eq!(at(6), (vec![1, 2], 1, 2), "a quarter clockwise: a above b");
        assert_eq!(
            at(8),
            (vec![2, 1], 1, 2),
            "a quarter the other way: b above a"
        );
    }
}
