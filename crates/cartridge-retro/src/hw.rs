//! Hardware rendering for a core that draws with OpenGL ES, as SwanStation's OpenGL renderer
//! does: the core gets a GLES context of its own on the thread it runs on, and a framebuffer to
//! draw each frame into, which is read back into the frame every other core hands over. So
//! rewind, save states, the looks and the thumbnails take its frames as they take any.
//!
//! The context stands beside the frontend's own, on the same EGL display, with a 16x16 pbuffer
//! to be current on: the core only ever draws into the framebuffer.

use std::ffi::{c_char, c_uint, c_void};
use std::sync::atomic::{AtomicPtr, Ordering};

use libloading::Library;

use crate::ffi::{
    HwRenderCallback, HW_CONTEXT_OPENGLES2, HW_CONTEXT_OPENGLES3, HW_CONTEXT_OPENGLES_VERSION,
};

type Egl = *mut c_void;
type EglInt = i32;

const EGL_NONE: EglInt = 0x3038;
const EGL_RENDERABLE_TYPE: EglInt = 0x3040;
const EGL_SURFACE_TYPE: EglInt = 0x3033;
const EGL_PBUFFER_BIT: EglInt = 0x0001;
const EGL_OPENGL_ES2_BIT: EglInt = 0x0004;
const EGL_OPENGL_ES3_BIT: EglInt = 0x0040;
const EGL_RED_SIZE: EglInt = 0x3024;
const EGL_GREEN_SIZE: EglInt = 0x3023;
const EGL_BLUE_SIZE: EglInt = 0x3022;
const EGL_WIDTH: EglInt = 0x3057;
const EGL_HEIGHT: EglInt = 0x3056;
const EGL_CONTEXT_MAJOR_VERSION: EglInt = 0x3098;
const EGL_CONTEXT_MINOR_VERSION: EglInt = 0x30FB;
const EGL_OPENGL_ES_API: c_uint = 0x30A0;

const GL_FRAMEBUFFER: c_uint = 0x8D40;
const GL_RENDERBUFFER: c_uint = 0x8D41;
const GL_TEXTURE_2D: c_uint = 0x0DE1;
const GL_RGBA: c_uint = 0x1908;
const GL_UNSIGNED_BYTE: c_uint = 0x1401;
const GL_COLOR_ATTACHMENT0: c_uint = 0x8CE0;
const GL_DEPTH_STENCIL_ATTACHMENT: c_uint = 0x821A;
const GL_DEPTH24_STENCIL8: c_uint = 0x88F0;
const GL_FRAMEBUFFER_COMPLETE: c_uint = 0x8CD5;
const GL_PACK_ALIGNMENT: c_uint = 0x0D05;

/// eglGetProcAddress, for `get_proc_address`, which a core may call from anywhere.
static PROC_ADDRESS: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

struct Fns {
    get_display: unsafe extern "C" fn(*mut c_void) -> Egl,
    initialize: unsafe extern "C" fn(Egl, *mut EglInt, *mut EglInt) -> c_uint,
    bind_api: unsafe extern "C" fn(c_uint) -> c_uint,
    choose_config:
        unsafe extern "C" fn(Egl, *const EglInt, *mut Egl, EglInt, *mut EglInt) -> c_uint,
    create_context: unsafe extern "C" fn(Egl, Egl, Egl, *const EglInt) -> Egl,
    create_pbuffer: unsafe extern "C" fn(Egl, Egl, *const EglInt) -> Egl,
    make_current: unsafe extern "C" fn(Egl, Egl, Egl, Egl) -> c_uint,
    destroy_context: unsafe extern "C" fn(Egl, Egl) -> c_uint,
    destroy_surface: unsafe extern "C" fn(Egl, Egl) -> c_uint,
    gen_framebuffers: unsafe extern "C" fn(i32, *mut c_uint),
    bind_framebuffer: unsafe extern "C" fn(c_uint, c_uint),
    gen_textures: unsafe extern "C" fn(i32, *mut c_uint),
    bind_texture: unsafe extern "C" fn(c_uint, c_uint),
    tex_image_2d:
        unsafe extern "C" fn(c_uint, i32, i32, i32, i32, i32, c_uint, c_uint, *const c_void),
    framebuffer_texture_2d: unsafe extern "C" fn(c_uint, c_uint, c_uint, c_uint, i32),
    gen_renderbuffers: unsafe extern "C" fn(i32, *mut c_uint),
    bind_renderbuffer: unsafe extern "C" fn(c_uint, c_uint),
    renderbuffer_storage: unsafe extern "C" fn(c_uint, c_uint, i32, i32),
    framebuffer_renderbuffer: unsafe extern "C" fn(c_uint, c_uint, c_uint, c_uint),
    check_framebuffer_status: unsafe extern "C" fn(c_uint) -> c_uint,
    pixel_storei: unsafe extern "C" fn(c_uint, i32),
    read_pixels: unsafe extern "C" fn(i32, i32, i32, i32, c_uint, c_uint, *mut c_void),
}

/// The core's GL context and the framebuffer it draws into.
pub struct HwGl {
    fns: Fns,
    display: Egl,
    context: Egl,
    surface: Egl,
    pub fbo: c_uint,
    size: (u32, u32),
    callback: HwRenderCallback,
    pixels: Vec<u8>,
    _egl: Library,
    _gles: Library,
}

unsafe fn sym<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, String> {
    lib.get::<T>(name)
        .map(|s| *s)
        .map_err(|e| format!("{}: {e}", String::from_utf8_lossy(name)))
}

fn open(names: &[&str]) -> Result<Library, String> {
    let mut last = String::new();
    for name in names {
        match unsafe { Library::new(name) } {
            Ok(lib) => return Ok(lib),
            Err(e) => last = format!("{name}: {e}"),
        }
    }
    Err(last)
}

/// The GLES version a core's callback asks for, (major, minor); None for anything but GLES.
pub fn gles_version(cb: &HwRenderCallback) -> Option<(i32, i32)> {
    match cb.context_type {
        HW_CONTEXT_OPENGLES2 => Some((2, 0)),
        HW_CONTEXT_OPENGLES3 => Some((3, 0)),
        HW_CONTEXT_OPENGLES_VERSION if cb.version_major <= 3 => {
            Some((cb.version_major as i32, cb.version_minor as i32))
        }
        _ => None,
    }
}

// The context is made when the core loads a game, which it does on the emulator's thread,
// and is only ever current there: the core is moved to that thread before it loads.
unsafe impl Send for HwGl {}

impl HwGl {
    /// A GLES context for `callback` made current on this thread, or why there is none.
    pub fn create(callback: HwRenderCallback) -> Result<HwGl, String> {
        let (major, minor) = gles_version(&callback).ok_or("not a GLES context")?;
        let egl = open(&["libEGL.so.1", "libEGL.so"])?;
        let gles = open(&["libGLESv2.so.2", "libGLESv2.so"])?;
        unsafe {
            let fns = Fns {
                get_display: sym(&egl, b"eglGetDisplay")?,
                initialize: sym(&egl, b"eglInitialize")?,
                bind_api: sym(&egl, b"eglBindAPI")?,
                choose_config: sym(&egl, b"eglChooseConfig")?,
                create_context: sym(&egl, b"eglCreateContext")?,
                create_pbuffer: sym(&egl, b"eglCreatePbufferSurface")?,
                make_current: sym(&egl, b"eglMakeCurrent")?,
                destroy_context: sym(&egl, b"eglDestroyContext")?,
                destroy_surface: sym(&egl, b"eglDestroySurface")?,
                gen_framebuffers: sym(&gles, b"glGenFramebuffers")?,
                bind_framebuffer: sym(&gles, b"glBindFramebuffer")?,
                gen_textures: sym(&gles, b"glGenTextures")?,
                bind_texture: sym(&gles, b"glBindTexture")?,
                tex_image_2d: sym(&gles, b"glTexImage2D")?,
                framebuffer_texture_2d: sym(&gles, b"glFramebufferTexture2D")?,
                gen_renderbuffers: sym(&gles, b"glGenRenderbuffers")?,
                bind_renderbuffer: sym(&gles, b"glBindRenderbuffer")?,
                renderbuffer_storage: sym(&gles, b"glRenderbufferStorage")?,
                framebuffer_renderbuffer: sym(&gles, b"glFramebufferRenderbuffer")?,
                check_framebuffer_status: sym(&gles, b"glCheckFramebufferStatus")?,
                pixel_storei: sym(&gles, b"glPixelStorei")?,
                read_pixels: sym(&gles, b"glReadPixels")?,
            };
            let proc_address: *mut c_void = sym(&egl, b"eglGetProcAddress")?;
            PROC_ADDRESS.store(proc_address, Ordering::Release);

            let display = (fns.get_display)(std::ptr::null_mut());
            if display.is_null()
                || (fns.initialize)(display, std::ptr::null_mut(), std::ptr::null_mut()) == 0
            {
                return Err("no EGL display".into());
            }
            (fns.bind_api)(EGL_OPENGL_ES_API);
            let bit = if major >= 3 {
                EGL_OPENGL_ES3_BIT
            } else {
                EGL_OPENGL_ES2_BIT
            };
            let want = [
                EGL_RENDERABLE_TYPE,
                bit,
                EGL_SURFACE_TYPE,
                EGL_PBUFFER_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_NONE,
            ];
            let mut config: Egl = std::ptr::null_mut();
            let mut found = 0;
            if (fns.choose_config)(display, want.as_ptr(), &mut config, 1, &mut found) == 0
                || found == 0
            {
                return Err(format!("no EGL config for GLES {major}"));
            }
            let attrs = [
                EGL_CONTEXT_MAJOR_VERSION,
                major,
                EGL_CONTEXT_MINOR_VERSION,
                minor,
                EGL_NONE,
            ];
            let context =
                (fns.create_context)(display, config, std::ptr::null_mut(), attrs.as_ptr());
            if context.is_null() {
                return Err(format!("no GLES {major}.{minor} context"));
            }
            let pb = [EGL_WIDTH, 16, EGL_HEIGHT, 16, EGL_NONE];
            let surface = (fns.create_pbuffer)(display, config, pb.as_ptr());
            if surface.is_null() || (fns.make_current)(display, surface, surface, context) == 0 {
                (fns.destroy_context)(display, context);
                return Err("the context would not be made current".into());
            }
            Ok(HwGl {
                fns,
                display,
                context,
                surface,
                fbo: 0,
                size: (0, 0),
                callback,
                pixels: Vec::new(),
                _egl: egl,
                _gles: gles,
            })
        }
    }

    /// The framebuffer the core draws into, `w` by `h` at the most, with the depth and stencil
    /// it asked for; made again only when it must grow.
    pub fn ensure_framebuffer(&mut self, w: u32, h: u32) -> Result<(), String> {
        if self.fbo != 0 && self.size.0 >= w && self.size.1 >= h {
            return Ok(());
        }
        unsafe {
            let f = &self.fns;
            let (mut fbo, mut tex) = (0, 0);
            (f.gen_framebuffers)(1, &mut fbo);
            (f.bind_framebuffer)(GL_FRAMEBUFFER, fbo);
            (f.gen_textures)(1, &mut tex);
            (f.bind_texture)(GL_TEXTURE_2D, tex);
            (f.tex_image_2d)(
                GL_TEXTURE_2D,
                0,
                GL_RGBA as i32,
                w as i32,
                h as i32,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null(),
            );
            (f.framebuffer_texture_2d)(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, tex, 0);
            if self.callback.depth || self.callback.stencil {
                let mut rb = 0;
                (f.gen_renderbuffers)(1, &mut rb);
                (f.bind_renderbuffer)(GL_RENDERBUFFER, rb);
                (f.renderbuffer_storage)(GL_RENDERBUFFER, GL_DEPTH24_STENCIL8, w as i32, h as i32);
                (f.framebuffer_renderbuffer)(
                    GL_FRAMEBUFFER,
                    GL_DEPTH_STENCIL_ATTACHMENT,
                    GL_RENDERBUFFER,
                    rb,
                );
            }
            if (f.check_framebuffer_status)(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                return Err(format!("framebuffer {w}x{h} incomplete"));
            }
            self.fbo = fbo;
            self.size = (w, h);
        }
        Ok(())
    }

    /// What tells the core its context is ready, for it to make its shaders and textures.
    pub fn reset_fn(&self) -> Option<crate::ffi::HwContextResetFn> {
        self.callback.context_reset
    }

    /// The frame the core drew, `w` by `h` from the framebuffer's corner, as XRGB8888 rows top
    /// down: GL's rows run up from the bottom unless the core drew them the other way.
    pub fn read(&mut self, w: u32, h: u32) -> Vec<u8> {
        let (w, h) = (w.min(self.size.0), h.min(self.size.1));
        let row = w as usize * 4;
        self.pixels.resize(row * h as usize, 0);
        unsafe {
            let f = &self.fns;
            (f.bind_framebuffer)(GL_FRAMEBUFFER, self.fbo);
            (f.pixel_storei)(GL_PACK_ALIGNMENT, 1);
            (f.read_pixels)(
                0,
                0,
                w as i32,
                h as i32,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                self.pixels.as_mut_ptr() as *mut c_void,
            );
        }
        if self.callback.bottom_left_origin {
            for y in 0..h as usize / 2 {
                let (top, bottom) = self.pixels.split_at_mut((h as usize - 1 - y) * row);
                top[y * row..(y + 1) * row].swap_with_slice(&mut bottom[..row]);
            }
        }
        // RGBA in memory to XRGB8888's B, G, R, X
        for px in self.pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        std::mem::take(&mut self.pixels)
    }

    /// The buffer `read` handed out, back for the next frame.
    pub fn give_back(&mut self, pixels: Vec<u8>) {
        self.pixels = pixels;
    }
}

impl Drop for HwGl {
    fn drop(&mut self) {
        unsafe {
            if let Some(destroy) = self.callback.context_destroy {
                destroy();
            }
            let f = &self.fns;
            (f.make_current)(
                self.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            (f.destroy_surface)(self.display, self.surface);
            (f.destroy_context)(self.display, self.context);
        }
    }
}

/// The core's `get_proc_address`: any GL or EGL function by name, through eglGetProcAddress,
/// which on this driver finds the core ones too.
pub unsafe extern "C" fn get_proc_address(name: *const c_char) -> *const c_void {
    let p = PROC_ADDRESS.load(Ordering::Acquire);
    if p.is_null() || name.is_null() {
        return std::ptr::null();
    }
    let f: unsafe extern "C" fn(*const c_char) -> *const c_void = std::mem::transmute(p);
    f(name)
}

/// Every 300 frames, with CARTRIDGE_TRACE set, how long reading a frame back took and how far
/// apart frames came: the read waits for the GPU to finish the frame.
pub fn note_read(took: std::time::Duration, w: u32, h: u32) {
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};
    static ON: OnceLock<bool> = OnceLock::new();
    if !*ON.get_or_init(|| std::env::var_os("CARTRIDGE_TRACE").is_some()) {
        return;
    }
    static STATS: Mutex<Option<(Instant, Duration, u32)>> = Mutex::new(None);
    let mut s = STATS.lock().unwrap_or_else(|e| e.into_inner());
    let (since, total, n) = s.get_or_insert((Instant::now(), Duration::ZERO, 0));
    *total += took;
    *n += 1;
    if *n == 300 {
        eprintln!(
            "cartridge: hw render: {w}x{h}, read back {:.2} ms a frame, {:.1} frames a second",
            total.as_secs_f64() * 1000.0 / 300.0,
            300.0 / since.elapsed().as_secs_f64()
        );
        *s = None;
    }
}
