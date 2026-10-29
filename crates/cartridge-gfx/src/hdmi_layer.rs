//! The framebuffer's scanout layer on the HDMI output. `System/hdmi.sh` switches the display
//! engine to the HDMI transmitter and resizes fb0 to the HDMI size, but the driver's fbdev glue has no
//! `set_par`: the pan offset the LCD left (480) stays, which shows the bottom of the picture
//! only, and the layer keeps the LCD's window and buffer size. [`settle`] zeroes the offset and
//! writes the layer for the new framebuffer and the 1920x1080 output, as NextUI does before its
//! apps start. It runs before the EGL surface exists.

use std::ffi::{c_int, c_ulong, c_void};
use std::time::Duration;

const ATTR_SYS: &str = "/sys/class/disp/disp/attr/sys";

/// The HDMI signal's size: the display engine scales fb0 up into it.
const OUT: (u32, u32) = (1920, 1080);

/// Where a framebuffer of `fb` goes on the output: the full height, as wide as its shape
/// takes, in the middle.
fn window(fb: (u32, u32)) -> Rect {
    let width = (u64::from(fb.0) * u64::from(OUT.1) / u64::from(fb.1.max(1))) as u32;
    let width = width.min(OUT.0);
    Rect {
        x: ((OUT.0 - width) / 2) as i32,
        y: 0,
        width,
        height: OUT.1,
    }
}

const O_RDWR: c_int = 2;
const FBIOGET_VSCREENINFO: c_ulong = 0x4600;
const FBIOPUT_VSCREENINFO: c_ulong = 0x4601;
const FB_ACTIVATE_NOW: u32 = 0;
const FB_ACTIVATE_FORCE: u32 = 128;
const DISP_LAYER_SET_CONFIG: c_ulong = 0x47;
const DISP_LAYER_GET_CONFIG: c_ulong = 0x48;

extern "C" {
    fn open(path: *const std::ffi::c_char, flags: c_int, ...) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

/// fb_var_screeninfo: 40 words, of which these are the ones used.
const X_RES: usize = 0;
const Y_RES: usize = 1;
const X_VIRTUAL: usize = 2;
const Y_VIRTUAL: usize = 3;
const X_OFFSET: usize = 4;
const Y_OFFSET: usize = 5;
const ACTIVATE: usize = 21;

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect64 {
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FbInfo {
    addr: [u64; 3],
    size: [Size; 3],
    align: [u32; 3],
    format: i32,
    color_space: i32,
    trd_right_addr: [u32; 3],
    pre_multiply: bool,
    crop: Rect64,
    flags: i32,
    scan: i32,
}

/// sunxi disp2's `struct disp_layer_info`, whose union of a colour and a framebuffer is the
/// framebuffer here.
#[repr(C)]
#[derive(Clone, Copy)]
struct LayerInfo {
    mode: i32,
    zorder: u8,
    alpha_mode: u8,
    alpha_value: u8,
    screen_win: Rect,
    b_trd_out: bool,
    out_trd_mode: i32,
    fb: FbInfo,
    id: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct LayerConfig {
    info: LayerInfo,
    enable: bool,
    channel: u32,
    layer_id: u32,
}

/// Whether the disp driver's status says the HDMI transmitter has the output.
pub fn hdmi_active(sys: &str) -> bool {
    sys.contains("hdmi output")
}

/// What the layer should hold for a framebuffer of `fb` (one page) on the HDMI output: the
/// whole double buffer as its size, page 0 as its crop, the full output as its window.
fn commit(config: &mut LayerConfig, fb: (u32, u32)) {
    config.enable = true;
    for size in &mut config.info.fb.size {
        *size = Size {
            width: fb.0,
            height: fb.1 * 2,
        };
    }
    config.info.fb.crop = Rect64 {
        x: 0,
        y: 0,
        width: i64::from(fb.0) << 32,
        height: i64::from(fb.1) << 32,
    };
    config.info.screen_win = window(fb);
}

fn layer_is_set(config: &LayerConfig, fb: (u32, u32)) -> bool {
    config.info.fb.size[0].width == fb.0
        && config.info.screen_win.width == window(fb).width
        && config.info.fb.crop.y == 0
}

/// Puts fb0 and its layer right for the HDMI output, if the output is HDMI. Best effort: a
/// failure leaves the picture where it was.
pub fn settle() {
    let active = std::fs::read_to_string(ATTR_SYS).is_ok_and(|text| hdmi_active(&text));
    if !active {
        return;
    }
    let Some(fb) = zero_pan() else {
        return;
    };
    commit_layer(fb);
}

/// [`settle`] again a moment after the EGL surface is up and once more later, on a thread of
/// its own: the driver puts the LCD's layer back as the display attaches and as the first frames
/// pan, and the last write wins.
pub fn settle_later() {
    if !std::fs::read_to_string(ATTR_SYS).is_ok_and(|text| hdmi_active(&text)) {
        return;
    }
    std::thread::spawn(|| {
        for ms in [300, 1200, 3000] {
            std::thread::sleep(Duration::from_millis(ms));
            settle();
        }
    });
}

/// Zeroes the pan offset with the geometry fb0 has, forcing the driver to take it. Returns one
/// page's size.
fn zero_pan() -> Option<(u32, u32)> {
    // SAFETY: plain ioctls on a descriptor we open and close here, with a 160-byte
    // fb_var_screeninfo buffer as the kernel's struct is
    unsafe {
        let fd = open(c"/dev/fb0".as_ptr(), O_RDWR);
        if fd < 0 {
            return None;
        }
        let mut var = [0u32; 40];
        let got = ioctl(fd, FBIOGET_VSCREENINFO, var.as_mut_ptr() as *mut c_void);
        let page = (var[X_RES], var[Y_RES]);
        let ok = got == 0 && page.0 > 0 && page.1 > 0;
        if ok {
            var[X_VIRTUAL] = page.0;
            var[Y_VIRTUAL] = page.1 * 2;
            var[X_OFFSET] = 0;
            var[Y_OFFSET] = 0;
            var[ACTIVATE] = FB_ACTIVATE_NOW | FB_ACTIVATE_FORCE;
            ioctl(fd, FBIOPUT_VSCREENINFO, var.as_mut_ptr() as *mut c_void);
        }
        close(fd);
        ok.then_some(page)
    }
}

/// Writes the fb0 layer (channel 1, layer 0) and reads it back until it holds: the driver
/// restores the previous mode's layer on attach, asynchronously, and the last write wins.
fn commit_layer(fb: (u32, u32)) {
    // SAFETY: the ioctls take `[0, &config, 1, 0]` and read or write one LayerConfig, laid out
    // as the kernel's struct disp_layer_config
    unsafe {
        let fd = open(c"/dev/disp".as_ptr(), O_RDWR);
        if fd < 0 {
            return;
        }
        for _ in 0..20 {
            let mut config: LayerConfig = std::mem::zeroed();
            config.channel = 1;
            config.layer_id = 0;
            let param = [0, &mut config as *mut LayerConfig as c_ulong, 1, 0];
            if ioctl(fd, DISP_LAYER_GET_CONFIG, param.as_ptr()) < 0 {
                break;
            }
            commit(&mut config, fb);
            if ioctl(fd, DISP_LAYER_SET_CONFIG, param.as_ptr()) < 0 {
                eprintln!("cartridge: hdmi: layer set refused");
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
            let mut back: LayerConfig = std::mem::zeroed();
            back.channel = 1;
            back.layer_id = 0;
            let param = [0, &mut back as *mut LayerConfig as c_ulong, 1, 0];
            if ioctl(fd, DISP_LAYER_GET_CONFIG, param.as_ptr()) < 0 || layer_is_set(&back, fb) {
                break;
            }
            eprintln!(
                "cartridge: hdmi: layer reads back {}x{} crop y {} for fb {}x{}",
                back.info.screen_win.width,
                back.info.screen_win.height,
                back.info.fb.crop.y,
                fb.0,
                fb.1
            );
        }
        close(fd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_driver_status_names_the_output() {
        assert!(hdmi_active("lcd output ...\nhdmi output mode(10)"));
        assert!(!hdmi_active("lcd output\tbacklight(165)"));
    }

    #[test]
    fn a_layer_is_set_when_it_shows_page_zero_on_the_whole_output() {
        let mut config: LayerConfig = unsafe { std::mem::zeroed() };
        config.info.fb.crop.y = 480;
        assert!(!layer_is_set(&config, (1440, 960)));
        commit(&mut config, (1440, 960));
        assert!(layer_is_set(&config, (1440, 960)));
        assert_eq!(config.info.fb.size[2].height, 1920);
        assert_eq!(config.info.screen_win.height, 1080);
    }

    #[test]
    fn the_window_is_full_height_and_centred() {
        let w = window((1440, 960));
        assert_eq!((w.x, w.y, w.width, w.height), (150, 0, 1620, 1080));
        let wide = window((1920, 1080));
        assert_eq!((wide.x, wide.width), (0, 1920));
    }
}
