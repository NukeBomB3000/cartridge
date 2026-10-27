//! What the programs cartridge starts on the handheld get from it. Mali's EGL opens the
//! framebuffer without close-on-exec, so wpa_supplicant, udhcpc and every other daemon wifi.sh
//! and bluetooth.sh left running held /dev/fb0 open long after cartridge let go of it; and a
//! dropbear it started again, after SSH was switched off and on, ran in cartridge's session
//! with SIGQUIT ignored, as cartridge has it. [`keep_files_to_self`] stops the first, and
//! [`as_a_daemon`] starts a program the way BaseOS's init scripts do.

use std::process::Command;

#[cfg(feature = "device")]
extern "C" {
    fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    fn setsid() -> i32;
    fn signal(signum: i32, handler: usize) -> usize;
    fn sigprocmask(how: i32, set: *const SigSet, old: *mut SigSet) -> i32;
}

/// glibc's sigset_t on aarch64: 1024 bits.
#[cfg(feature = "device")]
#[repr(C)]
struct SigSet([u64; 16]);

#[cfg(feature = "device")]
const F_GETFD: i32 = 1;
#[cfg(feature = "device")]
const F_SETFD: i32 = 2;
#[cfg(feature = "device")]
const FD_CLOEXEC: i32 = 1;
#[cfg(feature = "device")]
const SIG_SETMASK: i32 = 2;
#[cfg(feature = "device")]
const SIG_DFL: usize = 0;
/// The last of the standard signals, every one of which a daemon starts with at its default.
#[cfg(feature = "device")]
const LAST_SIGNAL: i32 = 31;

/// Marks every file cartridge has open, but its three standard streams, to close when it runs
/// another program: what the libraries it loads opened without saying so (Mali's EGL and the
/// framebuffer above all) stays cartridge's own. Called once the screen is up.
pub fn keep_files_to_self() {
    #[cfg(feature = "device")]
    {
        let Ok(dir) = std::fs::read_dir("/proc/self/fd") else {
            return;
        };
        // collected first, as reading the folder opens one more
        let fds: Vec<i32> = dir
            .flatten()
            .filter_map(|entry| entry.file_name().to_str()?.parse().ok())
            .filter(|&fd| fd > 2)
            .collect();
        for fd in fds {
            // SAFETY: fcntl on a descriptor number only reads or sets its flags; one that has
            // closed since it was listed answers -1, which is left alone
            unsafe {
                let flags = fcntl(fd, F_GETFD);
                if flags >= 0 && flags & FD_CLOEXEC == 0 {
                    fcntl(fd, F_SETFD, flags | FD_CLOEXEC);
                }
            }
        }
    }
}

/// `command` set to start as BaseOS's init scripts start a daemon: in a session of its own,
/// with no signal blocked and every one at its default, rather than with what cartridge's
/// thread had (it ignores SIGQUIT and SIGPIPE).
pub fn as_a_daemon(command: &mut Command) -> &mut Command {
    #[cfg(feature = "device")]
    {
        use std::os::unix::process::CommandExt;

        // SAFETY: between fork and exec the closure calls only setsid, signal and sigprocmask,
        // which are async-signal-safe, and allocates nothing
        unsafe {
            command.pre_exec(|| {
                setsid();
                for signum in 1..=LAST_SIGNAL {
                    // SIGKILL and SIGSTOP refuse, which is as it should be
                    signal(signum, SIG_DFL);
                }
                let none = SigSet([0; 16]);
                sigprocmask(SIG_SETMASK, &none, std::ptr::null_mut());
                Ok(())
            });
        }
    }
    command
}
