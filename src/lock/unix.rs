use crate::config::CONFIG_DIR;
use anyhow::anyhow;
use lazy_static::lazy_static;
use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{close, read, write};
use nix::unistd::{mkfifo, unlink};
use std::os::unix::io::RawFd;
use std::path::PathBuf;

lazy_static! {
    static ref IPC_PATH: PathBuf = CONFIG_DIR.join("ipc");
}

pub(super) struct Lock {
    fd: Option<RawFd>,
}

impl Lock {
    pub fn acquire() -> Option<Self> {
        match mkfifo(IPC_PATH.as_path(), Mode::S_IWUSR | Mode::S_IRUSR) {
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
        match open(
            IPC_PATH.as_path(),
            OFlag::O_WRONLY | OFlag::O_NONBLOCK,
            Mode::empty(),
        ) {
            Ok(fd) => {
                close(fd).ok(); // A reading process exists
                None
            }
            Err(Errno::ENXIO) => {
                log::info!(
                    "Acquired lock (no readers exist) on {}",
                    IPC_PATH.to_str().unwrap()
                );
                let fd = open(
                    IPC_PATH.as_path(),
                    OFlag::O_RDONLY | OFlag::O_NONBLOCK,
                    Mode::empty(),
                )
                .map_err(|e| {
                    log::warn!(
                        "Failed to open {} for reading: {}",
                        IPC_PATH.to_str().unwrap(),
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

    pub fn should_show_console(&self) -> bool {
        self.fd
            .as_ref()
            .map(|fd| {
                let mut buffer = vec![0 as u8; 1];
                match read(*fd, buffer.as_mut_slice()) {
                    Ok(r) => r > 0,
                    Err(_) => false,
                }
            })
            .unwrap_or(false)
    }

    pub fn show_console_in_owning_process() -> Result<(), anyhow::Error> {
        let fd = open(
            IPC_PATH.as_path(),
            OFlag::O_WRONLY | OFlag::O_NONBLOCK,
            Mode::empty(),
        )
        .map_err(|e| anyhow!("Opening pipe failed with: {}", e))?;
        write(fd, vec![0].as_slice()).map_err(|e| anyhow!("Writing to pipe failed with: {}", e))?;
        close(fd).ok();
        Ok(())
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        self.fd.as_ref().map(|fd| {
            close(*fd).ok();
            unlink(IPC_PATH.as_path()).ok();
        });
    }
}
