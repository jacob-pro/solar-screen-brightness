use crate::controller::worker;
use nix::poll::{poll, PollFd, PollFlags};
use nix::unistd::{close, pipe, write};
use std::ffi::CString;
use std::os::unix::prelude::{AsRawFd, RawFd};
use std::sync::mpsc::SyncSender;
use udev::ffi::{udev_device_get_action, udev_monitor_receive_device};
use udev::{AsRaw, MonitorBuilder};

// Linux Only:
// Monitors for new DDC/CI Connections
pub struct Monitor {
    fd: Option<RawFd>,
}

fn monitor_connections(read: RawFd, worker: SyncSender<worker::Message>) {
    if let Err(e) = (|| -> anyhow::Result<()> {
        let socket = MonitorBuilder::new()?.match_subsystem("ddcci")?.listen()?;
        log::info!("Monitoring for DDC/CI connections");
        loop {
            let socket_fd = PollFd::new(socket.as_raw_fd(), PollFlags::POLLIN);
            let stop = PollFd::new(read, PollFlags::POLLIN);
            let mut pfds = vec![socket_fd, stop];
            log::trace!("Beginning udev monitor connections poll...");
            poll(pfds.as_mut_slice(), -1)?;
            if let Some(e) = pfds[1].revents() {
                if e.contains(PollFlags::POLLIN) {
                    log::info!("DDC/CI Udev Monitor stopping");
                    close(read).ok();
                    break;
                }
            }
            if let Some(e) = pfds[0].revents() {
                if e.contains(PollFlags::POLLIN) {
                    let action = unsafe {
                        let dev = udev_monitor_receive_device(socket.as_raw());
                        let raw = udev_device_get_action(dev);
                        let cs = CString::from_raw(raw as *mut _);
                        cs.to_str()?.to_owned()
                    };
                    if action == "add" {
                        log::info!("Notified of ddcci add event, triggering refresh");
                        worker.send(worker::Message::ForceRefresh).ok();
                    }
                }
            }
        }
        Ok(())
    })() {
        log::info!("Failed monitoring for DDC/CI connections: {:#}", e);
    }
}

impl Monitor {
    pub fn start(worker: SyncSender<worker::Message>) -> Self {
        let fd = match pipe() {
            Err(e) => {
                log::info!("Failed setting up monitor pipe: {:#}", e);
                None
            }
            Ok((read, write)) => {
                std::thread::spawn(move || monitor_connections(read, worker));
                Some(write)
            }
        };
        Self { fd }
    }

    pub fn stop(&self) {
        if let Some(fd) = self.fd.as_ref() {
            write(*fd, &[0]).ok();
            close(*fd).ok();
        }
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop();
    }
}
