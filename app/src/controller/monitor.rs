use crate::controller::worker;
use std::sync::mpsc::SyncSender;

#[cfg(target_os = "linux")]
fn watch_ddcci_add(weak: Weak<RwLock<State>>) {
    use nix::poll::{poll, PollFd, PollFlags};
    use std::ffi::CString;
    use std::os::unix::prelude::AsRawFd;
    use udev::ffi::{udev_device_get_action, udev_monitor_receive_device};
    use udev::{AsRaw, MonitorBuilder};
    std::thread::spawn(move || {
        if let Err(e) = (|| -> anyhow::Result<()> {
            let listener = MonitorBuilder::new()?.match_subsystem("ddcci")?.listen()?;
            log::info!("Watching for monitor connections");
            loop {
                let pfd = PollFd::new(listener.as_raw_fd(), PollFlags::POLLIN);
                poll(&mut [pfd], -1)?;
                let action = unsafe {
                    let dev = udev_monitor_receive_device(listener.as_raw());
                    let raw = udev_device_get_action(dev);
                    let cs = CString::from_raw(raw as *mut _);
                    cs.to_str()?.to_owned()
                };
                match weak.upgrade() {
                    None => break,
                    Some(controller) => {
                        if action == "add" {
                            log::error!("Notified of ddcci add event, triggering refresh");
                            // TODO:
                        }
                    }
                }
            }
            Ok(())
        })() {
            log::error!("Error occurred watching for monitor connections {:#}", e);
        }
    });
}

// Linux Only:
// Monitors for DDC/CI Connections
pub struct Monitor {}

impl Monitor {
    pub fn start(worker: SyncSender<worker::Message>) -> Self {
        std::thread::spawn(move || loop {
            worker.send(worker::Message::ForceRefresh).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(10));
        });
        Self {}
    }

    fn stop(&self) {}
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop();
    }
}
