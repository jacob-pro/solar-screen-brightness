use crate::event_watcher::linux::{get_ipc_path, MSG_OPEN_WINDOW};
use crate::unique::Error;
use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{close, write};
use nix::unistd::{mkfifo, unlink};
use std::os::unix::io::RawFd;

#[derive(Debug, Error)]
enum PlatformError {
    #[error("Failed to call mkfifo: {0}")]
    Mkfifo(#[source] Errno),
    #[error("Failed to open FIFO for writing: {0}")]
    OpenWrite(#[source] Errno),
    #[error("Failed to open FIFO for reading: {0}")]
    OpenRead(#[source] Errno),
}

// A handle to a FIFO reader, indicating that this is the unique instance of SSB
pub struct SsbUniqueInstance {
    fd: RawFd,
}

impl SsbUniqueInstance {
    pub fn try_acquire() -> Result<SsbUniqueInstance, Error> {
        let ipc_path = get_ipc_path();
        // Create the FIFO special file, ignore if it already exists
        match mkfifo(&ipc_path, Mode::S_IWUSR | Mode::S_IRUSR) {
            Ok(_) => {}
            Err(Errno::EEXIST) => {}
            Err(e) => return Err(Error::PlatformError(Box::new(PlatformError::Mkfifo(e)))),
        }
        // Attempt to open the FIFO for writing, this will only succeed if there is an existing
        // instance reading on this FIFO.
        match open(
            &ipc_path,
            OFlag::O_WRONLY | OFlag::O_NONBLOCK,
            Mode::empty(),
        ) {
            // Success means that a reading process exists
            Ok(fd) => Err(Error::AlreadyRunning(ExistingInstance(fd))),
            // ENXIO means that there is no process reading from this FIFO
            Err(Errno::ENXIO) => {
                // We must create a reader to hold the lock
                // We must use non block otherwise it will block until a writer is connected
                match open(
                    &ipc_path,
                    OFlag::O_RDONLY | OFlag::O_NONBLOCK,
                    Mode::empty(),
                ) {
                    Ok(fd) => Ok(SsbUniqueInstance { fd }),
                    Err(e) => Err(Error::PlatformError(Box::new(PlatformError::OpenRead(e)))),
                }
            }
            Err(e) => Err(Error::PlatformError(Box::new(PlatformError::OpenWrite(e)))),
        }
    }
}

impl Drop for SsbUniqueInstance {
    fn drop(&mut self) {
        close(self.fd).unwrap();
        unlink(&get_ipc_path()).ok();
    }
}

#[derive(Debug)]
pub struct ExistingInstance(RawFd);

/// Represents an already running instance of Solar Screen Brightness
impl ExistingInstance {
    /// Writes the MSG_OPEN_WINDOW down the IPC pipe
    pub fn wakeup(&self) {
        write(self.0, vec![MSG_OPEN_WINDOW].as_slice()).unwrap();
    }
}

impl Drop for ExistingInstance {
    fn drop(&mut self) {
        close(self.0).unwrap();
    }
}
