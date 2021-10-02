pub mod apply;
mod worker;

use crate::config::Config;
use crate::controller::apply::ApplyResult;
use crate::controller::worker::Worker;
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, RwLock, Weak};

pub type DelegateImpl = Weak<dyn Delegate + Send + Sync>;

pub trait Delegate {
    fn did_set_enabled(&self, _enabled: bool) {}
    fn did_set_last_result(&self, _last_result: &ApplyResult) {}
    fn did_set_config(&self, _config: &Config) {}
}

pub struct BrightnessController {
    config: RwLock<Config>,
    enabled: RwLock<bool>,
    last_result: Arc<RwLock<ApplyResult>>,
    worker: Arc<RwLock<Option<SyncSender<worker::Message>>>>,
    delegate: Arc<RwLock<DelegateImpl>>,
}

// https://users.rust-lang.org/t/why-cant-weak-new-be-used-with-a-trait-object/29976/14
struct DummyDelegate;
impl Delegate for DummyDelegate {}

impl BrightnessController {
    pub fn new(config: Config) -> Self {
        let delegate: Weak<DummyDelegate> = Weak::new();
        Self {
            config: RwLock::new(config),
            enabled: RwLock::new(true),
            last_result: Arc::new(RwLock::new(ApplyResult::None)),
            worker: Arc::new(RwLock::new(None)),
            delegate: Arc::new(RwLock::new(delegate as DelegateImpl)),
        }
    }

    pub fn start(&self) {
        let mut worker = self.worker.write().unwrap();
        if worker.is_none() {
            log::info!("Starting BrightnessController");
            let config = self.config.read().unwrap().clone();
            let enabled = *self.enabled.read().unwrap();
            let delegate = Arc::clone(&self.delegate);
            let last_result = Arc::clone(&self.last_result);

            let sender = Worker::start(config, enabled, move |res| {
                let mut last_result_rw = last_result.write().unwrap();
                let delegate_r = delegate.read().unwrap();
                *last_result_rw = res;
                delegate_r
                    .upgrade()
                    .map(|d| d.did_set_last_result(&*last_result_rw));
            });
            *worker = Some(sender);

            // watch_ddcci_add(Arc::downgrade(&self.0));
        } else {
            log::warn!("BrightnessController is already running, ignoring");
        }
    }

    pub fn stop(&self) {
        let mut worker = self.worker.write().unwrap();
        let worker = std::mem::take(&mut *worker);
        match worker {
            None => {}
            Some(tx) => {
                log::info!("Stopping Brightness Worker");
                tx.send(worker::Message::Terminate).unwrap();
            }
        }
    }

    #[allow(unused)]
    pub fn is_running(&self) -> bool {
        self.worker.read().unwrap().is_some()
    }

    pub fn get_enabled(&self) -> bool {
        *self.enabled.read().unwrap()
    }

    pub fn get_config(&self) -> Config {
        self.config.read().unwrap().clone()
    }

    pub fn get_last_result(&self) -> ApplyResult {
        self.last_result.read().unwrap().clone()
    }

    pub fn set_delegate(&self, delegate: DelegateImpl) {
        *self.delegate.write().unwrap() = delegate;
    }

    /// Enable or disable solar screen brightness, returns the previous value
    pub fn set_enabled(&self, enabled: bool) -> bool {
        let mut enabled_rw = self.enabled.write().unwrap();
        let worker_r = self.worker.read().unwrap();
        let delegate_r = self.delegate.read().unwrap();

        if enabled {
            log::info!("Enabling dynamic brightness");
        } else {
            log::info!("Disabling dynamic brightness");
        }
        let before = std::mem::replace(&mut *enabled_rw, enabled);
        worker_r
            .as_ref()
            .map(|w| w.send(worker::Message::UpdateEnabled(enabled)).unwrap());
        delegate_r.upgrade().map(|d| d.did_set_enabled(enabled));
        before
    }

    /// Update the solar screen brightness config, returns the previous config
    pub fn set_config(&self, config: Config) -> Config {
        log::info!("Applying new config");
        let mut config_rw = self.config.write().unwrap();
        let worker_r = self.worker.read().unwrap();
        let delegate_r = self.delegate.read().unwrap();

        let before = std::mem::replace(&mut *config_rw, config);
        worker_r.as_ref().map(|w| {
            w.send(worker::Message::UpdateConfig(config_rw.clone()))
                .unwrap()
        });
        delegate_r.upgrade().map(|d| d.did_set_config(&*config_rw));
        before
    }
}

impl Drop for BrightnessController {
    fn drop(&mut self) {
        self.stop();
    }
}

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

// #[cfg(not(target_os = "linux"))]
// fn watch_ddcci_add(_: Weak<RwLock<State>>) {}
