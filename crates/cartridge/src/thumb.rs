/// A save state's screenshot: the picture `size` pixels wide and high, XRGB8888, as a PNG of the
/// same size. A SNES or PlayStation picture keeps its own size, a Game Boy's the GBA screen it is
/// centred in.
pub fn png(xrgb8888: &[u8], size: (u32, u32)) -> Option<Vec<u8>> {
    let (w, h) = size;
    let n = (w * h) as usize;
    if n == 0 || xrgb8888.len() < n * 4 {
        return None;
    }
    let mut rgb = Vec::with_capacity(n * 3);
    for px in xrgb8888[..n * 4].chunks_exact(4) {
        rgb.extend_from_slice(&[px[2], px[1], px[0]]);
    }

    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().ok()?;
    writer.write_image_data(&rgb).ok()?;
    writer.finish().ok()?;
    Some(out)
}
