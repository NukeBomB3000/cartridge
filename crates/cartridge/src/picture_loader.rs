//! The pictures the viewer shows: each decoded on a thread of its own, no bigger than the screen,
//! and handed on as XRGB8888 through the same frames the games and videos use. A newer request
//! drops an older one still being decoded, so paging quickly through a folder shows the last.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::frames::{FrameRef, Frames};

pub struct Loader {
    frames: Arc<Frames>,
    latest: Arc<AtomicU64>,
    shown: Arc<AtomicBool>,
    room: (u32, u32),
}

/// `rgba`, `sw` by `sh`, as XRGB8888 (blue first) `dw` by `dh`, by averaging each pixel's patch of
/// the source: a picture is never made bigger than it is here, only smaller.
pub fn xrgb_scaled(rgba: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for y in 0..dh {
        let (y0, y1) = (
            y * sh / dh,
            ((y + 1) * sh / dh).max(y * sh / dh + 1).min(sh),
        );
        for x in 0..dw {
            let (x0, x1) = (
                x * sw / dw,
                ((x + 1) * sw / dw).max(x * sw / dw + 1).min(sw),
            );
            let mut acc = [0u32; 3];
            let mut n = 0u32;
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let i = ((sy * sw + sx) * 4) as usize;
                    acc[0] += u32::from(rgba[i]);
                    acc[1] += u32::from(rgba[i + 1]);
                    acc[2] += u32::from(rgba[i + 2]);
                    n += 1;
                }
            }
            let o = ((y * dw + x) * 4) as usize;
            out[o] = (acc[2] / n) as u8;
            out[o + 1] = (acc[1] / n) as u8;
            out[o + 2] = (acc[0] / n) as u8;
            out[o + 3] = 255;
        }
    }
    out
}

/// The size a picture `sw` by `sh` is shown at in `room`: its own if it fits, else as big as the
/// room holds in its shape.
pub fn fit_size(sw: u32, sh: u32, room: (u32, u32)) -> (u32, u32) {
    if sw <= room.0 && sh <= room.1 {
        return (sw.max(1), sh.max(1));
    }
    let scale = (room.0 as f32 / sw as f32).min(room.1 as f32 / sh as f32);
    (
        ((sw as f32 * scale).round() as u32).max(1),
        ((sh as f32 * scale).round() as u32).max(1),
    )
}

impl Loader {
    pub fn new(room: (u32, u32)) -> Loader {
        Loader {
            frames: Frames::new((room.0 * room.1 * 4) as usize),
            latest: Arc::new(AtomicU64::new(0)),
            shown: Arc::new(AtomicBool::new(false)),
            room,
        }
    }

    /// Asks for `path` to be decoded and shown.
    pub fn load(&self, path: PathBuf) {
        let id = self.latest.fetch_add(1, Ordering::SeqCst) + 1;
        let (frames, latest, shown, room) = (
            self.frames.clone(),
            self.latest.clone(),
            self.shown.clone(),
            self.room,
        );
        std::thread::spawn(move || {
            let Some((rgba, sw, sh)) = cartridge_ui::decode_picture(&path, room) else {
                return;
            };
            // a newer request has come in while this decoded: this one is not wanted
            if latest.load(Ordering::SeqCst) != id {
                return;
            }
            let (dw, dh) = fit_size(sw, sh, room);
            let buf = xrgb_scaled(&rgba, sw, sh, dw, dh);
            if latest.load(Ordering::SeqCst) == id {
                frames.publish(buf, (dw, dh));
                shown.store(true, Ordering::Relaxed);
            }
        });
    }

    pub fn frame(&self) -> Option<FrameRef> {
        self.frames.latest()
    }

    /// A picture has been shown.
    pub fn published(&self) -> bool {
        self.shown.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_that_fits_keeps_its_size_and_a_big_one_is_fitted_in_its_shape() {
        assert_eq!(fit_size(320, 200, (640, 480)), (320, 200));
        assert_eq!(fit_size(6000, 4000, (640, 480)), (640, 427));
        assert_eq!(fit_size(4000, 6000, (640, 480)), (320, 480));
    }

    #[test]
    fn scaling_down_averages_and_puts_blue_first() {
        // 2 by 2: two red pixels over two blue ones, averaged to one purple pixel
        let rgba = [
            255, 0, 0, 255, 255, 0, 0, 255, 0, 0, 255, 255, 0, 0, 255, 255,
        ];
        let out = xrgb_scaled(&rgba, 2, 2, 1, 1);
        assert_eq!(out, vec![127, 0, 127, 255], "blue, green, red, opaque");
        // the same size: only the order of the colours changes
        assert_eq!(
            xrgb_scaled(&[10, 20, 30, 255], 1, 1, 1, 1),
            vec![30, 20, 10, 255]
        );
    }
}
