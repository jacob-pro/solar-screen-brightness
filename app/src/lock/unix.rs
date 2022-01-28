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

const MSG_SHOW_CONSOLE: u8 = 1;
const MSG_STOP_WATCHING: u8 = 2;

pub fn acquire() -> Result<Lock, Existing> {
    match mkfifo(IPC_PATH.as_path(), Mode::S_IWUSR | Mode::S_IRUSR) {
        Ok(_) => {}
        Err(Errno::EEXIST) => {}
        Err(e) => {
            log::warn!(
                "Unexpected error acquiring lock: mkfifo() {}, ignoring with dummy lock",
                e
            );
            return Ok(Lock { fd: None });
        }
    }
    match open(
        IPC_PATH.as_path(),
        OFlag::O_WRONLY | OFlag::O_NONBLOCK,
        Mode::empty(),
    ) {
        Ok(fd) => {
            Err(Existing(fd)) // Success means that a reading process exists
        }
        Err(Errno::ENXIO) => {
            log::info!(
                "Acquired lock (no readers exist) on {}",
                IPC_PATH.to_str().unwrap()
            );
            // We must create a reader to hold the lock
            // We must use non block otherwise it will block until a writer is connected
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
            Ok(Lock { fd })
        }
        Err(e) => {
            log::warn!(
                "Unexpected error acquiring lock: open() {}, ignoring with dummy lock",
                e
            );
            Ok(Lock { fd: None })
        }
    }
}

pub struct Lock {
    fd: Option<RawFd>,
}

impl Drop for Lock {
    fn drop(&mut self) {
        if let Some(fd) = self.fd {
            close(fd).ok();
            unlink(IPC_PATH.as_path()).ok();
        }
    }
}

pub struct Existing(RawFd);

impl Existing {
    pub fn show_console(&self) -> std::result::Result<(), anyhow::Error> {
        write(self.0, vec![MSG_SHOW_CONSOLE].as_slice())
            .map_err(|e| anyhow!("Writing to pipe failed with: {}", e))?;
        Ok(())
    }
}

impl Drop for Existing {
    fn drop(&mut self) {
        close(self.0).ok();
    }
}

pub struct ShowConsoleWatcher();

impl ShowConsoleWatcher {
    pub fn start<T>(action: T) -> Self
    where
        T: Fn() + Send + 'static,
    {
        std::thread::spawn(move || {
            if let Err(e) = (|| -> anyhow::Result<()> {
                'outer: loop {
                    let fd = open(IPC_PATH.as_path(), OFlag::O_RDONLY, Mode::empty())?;
                    let mut buffer = vec![0_u8; 1];
                    loop {
                        let len = read(fd, buffer.as_mut_slice())?;
                        if len == 0 {
                            break; // Occurs when the writer disconnects, reopen the file and wait again
                        }
                        match buffer[0] {
                            MSG_SHOW_CONSOLE => action(),
                            MSG_STOP_WATCHING => {
                                log::info!("ShowConsoleWatcher is stopping");
                                close(fd).ok();
                                break 'outer;
                            }
                            _ => {}
                        }
                    }
                    close(fd).ok();
                }
                Ok(())
            })() {
                log::info!("Failed watching for show console requests: {:#}", e);
            }
        });
        Self()
    }
}

impl Drop for ShowConsoleWatcher {
    fn drop(&mut self) {
        if let Ok(fd) = open(IPC_PATH.as_path(), OFlag::O_WRONLY, Mode::empty()) {
            write(fd, vec![MSG_STOP_WATCHING].as_slice()).ok();
            close(fd).ok();
        }
    }
}
