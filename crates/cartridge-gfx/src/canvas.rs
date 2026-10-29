use std::sync::OnceLock;

/// The size everything is laid out and composed at, before the one blit to the panel. A device
/// gets the canvas its panel is shaped like, chosen once at boot.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Canvas {
    pub w: u32,
    pub h: u32,
}

impl Canvas {
    /// The RG SP, RG34XX and RG34XX SP.
    pub const WIDE: Canvas = Canvas { w: 720, h: 480 };
    /// The RG35XX family, the RG40XX H and V, and the RG28XX turned on its side.
    pub const FOUR_THREE: Canvas = Canvas { w: 640, h: 480 };
    /// The RG CubeXX.
    pub const SQUARE: Canvas = Canvas { w: 720, h: 720 };
    /// A 16:9 monitor on HDMI: half of 1080p, which the display engine doubles exactly.
    pub const MONITOR: Canvas = Canvas { w: 960, h: 540 };
    pub const ALL: [Canvas; 4] = [
        Canvas::WIDE,
        Canvas::FOUR_THREE,
        Canvas::SQUARE,
        Canvas::MONITOR,
    ];

    /// One of the canvases from `WxH`, such as `640x480`.
    pub fn parse(s: &str) -> Option<Canvas> {
        let (w, h) = s.trim().split_once(['x', 'X'])?;
        let c = Canvas {
            w: w.trim().parse().ok()?,
            h: h.trim().parse().ok()?,
        };
        Canvas::ALL.contains(&c).then_some(c)
    }

    /// The canvas for a panel of `w` by `h`, held the way it is read: the one that size, or
    /// failing that the one closest to its shape.
    pub fn for_panel(w: u32, h: u32) -> Canvas {
        if let Some(c) = Canvas::ALL.iter().find(|c| (c.w, c.h) == (w, h)) {
            return *c;
        }
        let aspect = w as f32 / h.max(1) as f32;
        if aspect >= 1.7 {
            Canvas::MONITOR
        } else if aspect >= 1.45 {
            Canvas::WIDE
        } else if aspect >= 1.2 {
            Canvas::FOUR_THREE
        } else {
            Canvas::SQUARE
        }
    }
}

pub const CANVAS_ENV: &str = "CARTRIDGE_CANVAS";

static CANVAS: OnceLock<Canvas> = OnceLock::new();

/// The canvas this process draws at. Unless the device set one at boot it is `CARTRIDGE_CANVAS`,
/// or the wide canvas the RG SP was first drawn on.
pub fn canvas() -> Canvas {
    *CANVAS.get_or_init(|| {
        std::env::var(CANVAS_ENV)
            .ok()
            .and_then(|v| Canvas::parse(&v))
            .unwrap_or(Canvas::WIDE)
    })
}

/// Fixes the canvas for the rest of the process. It can only be set before anything has
/// drawn: once it has been read, asking for a different one returns the canvas in use.
pub fn set_canvas(c: Canvas) -> Result<(), Canvas> {
    match CANVAS.set(c) {
        Ok(()) => Ok(()),
        Err(_) if canvas() == c => Ok(()),
        Err(_) => Err(canvas()),
    }
}

pub fn out_w() -> u32 {
    canvas().w
}

pub fn out_h() -> u32 {
    canvas().h
}
