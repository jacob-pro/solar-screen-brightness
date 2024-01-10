use crate::common::local_data_directory;
use crate::controller::{BrightnessController, Message};
use crate::gui::UserEvent;
use egui_winit::winit::event_loop::{EventLoop, EventLoopProxy};
use nix::fcntl::{open, OFlag};
use nix::poll::{poll, PollFd, PollFlags};
use nix::sys::stat::Mode;
use nix::unistd::{close, pipe, read, write};
use std::ffi::CString;
use std::os::unix::prelude::{AsRawFd, RawFd};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::JoinHandle;
use udev::ffi::{udev_device_get_action, udev_monitor_receive_device};
use udev::{AsRaw, MonitorBuilder};

pub const MSG_OPEN_WINDOW: u8 = 1;
const MSG_STOP_WATCHING: u8 = 2;

pub fn get_ipc_path() -> PathBuf {
    local_data_directory().join("ipc")
}

pub struct EventWatcher {
    ddcci_write_end: RawFd,
    ddcci_watcher_thread: Option<JoinHandle<()>>,
    ipc_path: PathBuf,
    ipc_watcher_thread: Option<JoinHandle<()>>,
}

/// Detect when DDC/CI monitors are added, tell the BrightnessController to refresh
fn watch_ddcci(read: RawFd, controller: mpsc::Sender<Message>) {
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
                        controller.send(Message::Refresh("udev add")).unwrap();
                    }
                }
            }
        }
        log::debug!("DDC/CI watcher thread exiting");
        Ok(())
    })() {
        log::error!("Error occurred monitoring for DDC/CI connections: {:#}", e);
    }
}

/// Listen to the IPC pipe for the MSG_OPEN_WINDOW
fn watch_ipc_pipe(ipc_path: PathBuf, event_loop: Option<EventLoopProxy<UserEvent>>) {
    'outer: loop {
        let fd = open(&ipc_path, OFlag::O_RDONLY, Mode::empty()).unwrap();
        let mut buffer = vec![0_u8; 1];
        loop {
            let len = read(fd, buffer.as_mut_slice()).unwrap();
            if len == 0 {
                break; // Occurs when the writer disconnects, reopen the file and wait again
            }
            match buffer[0] {
                MSG_OPEN_WINDOW => {
                    if let Some(event_loop) = &event_loop {
                        event_loop
                            .send_event(UserEvent::OpenWindow("IPC watcher"))
                            .unwrap();
                    }
                }
                MSG_STOP_WATCHING => {
                    close(fd).unwrap();
                    break 'outer;
                }
                _ => {}
            }
        }
        close(fd).unwrap();
    }
    log::debug!("IPC watcher thread exiting");
}

impl EventWatcher {
    pub fn start(
        controller: &BrightnessController,
        event_loop: Option<&EventLoop<UserEvent>>,
    ) -> anyhow::Result<Self> {
        let sender = controller.sender.clone();
        let (read, write) = pipe().unwrap();
        let ddcci_watcher_thread = std::thread::spawn(move || watch_ddcci(read, sender));

        let ipc_path = get_ipc_path();
        let proxy = event_loop.map(|e| e.create_proxy());
        let ipc_path2 = ipc_path.clone();
        let ipc_watcher_thread = std::thread::spawn(move || watch_ipc_pipe(ipc_path2, proxy));

        Ok(Self {
            ddcci_write_end: write,
            ddcci_watcher_thread: Some(ddcci_watcher_thread),
            ipc_path,
            ipc_watcher_thread: Some(ipc_watcher_thread),
        })
    }
}

impl Drop for EventWatcher {
    fn drop(&mut self) {
        log::info!("Stopping DDC/CI watcher");
        write(self.ddcci_write_end, &[0]).unwrap();
        close(self.ddcci_write_end).unwrap();
        self.ddcci_watcher_thread.take().unwrap().join().unwrap();

        log::info!("Stopping IPC watcher");
        let ipc_fd = open(&self.ipc_path, OFlag::O_WRONLY, Mode::empty()).unwrap();
        write(ipc_fd, vec![MSG_STOP_WATCHING].as_slice()).ok();
        close(ipc_fd).unwrap();
        self.ipc_watcher_thread.take().unwrap().join().unwrap();
    }
}
