use crate::config::CONFIG_DIR;
use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use nix::unistd::{close, write};
use std::os::unix::io::RawFd;

pub(super) struct Lock {
    fd: Option<RawFd>,
}

impl Lock {
    pub fn acquire() -> Option<Self> {
        let path = CONFIG_DIR.join("ipc");
        match mkfifo(&path, Mode::S_IWUSR | Mode::S_IRUSR) {
            Ok(_) => {}
            Err(Errno::EEXIST) => {}
            Err(e) => {
                log::warn!(
                    "Unexpected error acquiring lock: mkfifo() {}, ignoring with dummy lock",
                    e
                );
                return Some(Lock { fd: None });
            }
        }
        match open(&path, OFlag::O_WRONLY | OFlag::O_NONBLOCK, Mode::empty()) {
            Ok(fd) => {
                close(fd).ok(); // A reading process exists
                None
            }
            Err(Errno::ENXIO) => {
                log::info!(
                    "Acquired lock (no readers exist) on {}",
                    path.to_str().unwrap()
                );
                let fd = open(&path, OFlag::O_RDONLY | OFlag::O_NONBLOCK, Mode::empty())
                    .map_err(|e| {
                        log::warn!(
                            "Failed to open {} for reading: {}",
                            path.to_str().unwrap(),
                            e
                        );
                    })
                    .ok();
                Some(Lock { fd })
            }
            Err(e) => {
                log::warn!(
                    "Unexpected error acquiring lock: open() {}, ignoring with dummy lock",
                    e
                );
                return Some(Lock { fd: None });
            }
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        self.fd.as_ref().map(|fd| {
            close(*fd).ok();
        });
    }
}
