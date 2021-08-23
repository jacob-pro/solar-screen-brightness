pub mod apply;

use crate::config::Config;
use crate::controller::apply::{apply, ApplyResult};
use chrono::prelude::*;
use std::sync::mpsc::{sync_channel, RecvTimeoutError, SyncSender};
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

pub trait Observer {
    fn did_set_enabled(&self, running: bool);
    fn did_set_last_result(&self, last_result: &ApplyResult);
    fn did_set_config(&self, config: &Config);
}

enum Notification {
    Refresh,
    Terminate,
}

#[derive(Clone)]
pub struct BrightnessController(Arc<RwLock<BrightnessControllerInner>>);

// The inner data is shared across threads
struct BrightnessControllerInner {
    last_result: ApplyResult,
    enabled: bool,
    config: Config,
    observers: Vec<Weak<dyn Observer + Send + Sync>>,
    tx: Option<SyncSender<Notification>>,
}

impl BrightnessController {

    pub fn new(config: Config) -> Self {
        Self(Arc::new(RwLock::new(BrightnessControllerInner {
            last_result: ApplyResult::None,
            enabled: true,
            config,
            observers: vec![],
            tx: None
        })))
    }

    pub fn start(&mut self) {
        let mut write = self.0.write().unwrap();
        if write.tx.is_none() {
            let (tx, rx) = sync_channel::<Notification>(0);
            write.tx = Some(tx);
            let this = self.0.clone();
            std::thread::spawn(move || {
                loop {
                    let (res, next_run) = apply(&this);
                    this.write().unwrap().set_last_result(res);

                    // Wait for the next run, or a notification
                    let unix_time_now = Utc::now().timestamp();
                    let wait = if next_run > unix_time_now {
                        next_run - unix_time_now
                    } else {
                        0
                    };
                    match rx.recv_timeout(Duration::from_secs(wait as u64)) {
                        Ok(msg) => match msg {
                            Notification::Refresh => {}
                            Notification::Terminate => break,
                        },
                        Err(e) => {
                            if e != RecvTimeoutError::Timeout {
                                panic!("{}", e)
                            }
                        }
                    };
                }
            });
        }
    }

    pub fn stop(&self) {
        self.0.write().unwrap().stop();
    }

    #[allow(unused)]
    pub fn is_running(&self) -> bool {
        self.0.read().unwrap().tx.is_some()
    }

    pub fn get_enabled(&self) -> bool {
        self.0.read().unwrap().enabled
    }

    pub fn get_config(&self) -> Config {
        self.0.read().unwrap().config.clone()
    }

    pub fn get_last_result(&self) -> ApplyResult {
        self.0.read().unwrap().last_result.clone()
    }

    pub fn register(&self, o: Weak<dyn Observer + Send + Sync>) {
        self.0.write().unwrap().register(o);
    }

    /// Enable or disable solar screen brightness, returns the previous value
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        self.0.write().unwrap().set_enabled(enabled)
    }

    /// Update the solar screen brightness config, returns the previous config
    pub fn set_config(&mut self, config: Config) -> Config {
        self.0.write().unwrap().set_config(config)
    }

}

impl BrightnessControllerInner {

    fn stop(&mut self) {
        let tx = std::mem::take(&mut self.tx);
        tx.map(|tx| {
            tx.send(Notification::Terminate).unwrap();
        });
    }

    fn register(&mut self, o: Weak<dyn Observer + Send + Sync>) {
        self.clean_observers();
        self.observers.push(o);
    }

    #[allow(unused)]
    fn unregister(&mut self, o: Weak<dyn Observer + Send + Sync>) {
        let observers = std::mem::take(&mut self.observers);
        self.observers = observers
            .into_iter()
            .filter(|o2| !o2.ptr_eq(&o) && o2.upgrade().is_some())
            .collect()
    }

    fn set_enabled(&mut self, enabled: bool) -> bool {
        let before = self.enabled;
        self.enabled = enabled;
        self.tx.as_ref().map(|tx| tx.send(Notification::Refresh).unwrap());
        self.clean_observers();
        self.notify_observers(|o| o.did_set_enabled(enabled));
        before
    }

    fn set_config(&mut self, config: Config) -> Config {
        let before = std::mem::replace(&mut self.config, config);
        self.tx.as_ref().map(|tx| tx.send(Notification::Refresh).unwrap());
        self.clean_observers();
        self.notify_observers(|o| o.did_set_config(&self.config));
        before
    }

    fn set_last_result(&mut self, last_result: ApplyResult) {
        self.last_result = last_result;
        self.clean_observers();
        self.notify_observers(|o| o.did_set_last_result(&self.last_result));
    }

    fn clean_observers(&mut self) {
        let observers = std::mem::take(&mut self.observers);
        self.observers = observers
            .into_iter()
            .filter(|p| p.upgrade().is_some())
            .collect();
    }

    fn notify_observers<F>(&self, f: F)
        where
            F: Fn(Arc<dyn Observer + Send + Sync>),
    {
        self.observers.iter().for_each(|p| match p.upgrade() {
            None => {}
            Some(p) => {
                f(p);
            }
        })
    }
}

impl Drop for BrightnessControllerInner {
    fn drop(&mut self) {
        self.stop();
    }
}
