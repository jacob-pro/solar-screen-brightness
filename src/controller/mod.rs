pub mod apply;
pub mod state;

use crate::config::Config;
use crate::controller::apply::{apply, ApplyResult};
use crate::controller::state::Observer;
use chrono::prelude::*;
use state::State;
use std::sync::mpsc::{sync_channel, RecvTimeoutError, SyncSender};
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

pub type StateRef = Arc<RwLock<State>>;

enum Notification {
    Refresh,
    Terminate,
}

pub struct BrightnessController {
    pub state: StateRef,
    state_watcher: Option<Arc<StateWatcher>>,
}

impl BrightnessController {
    pub fn new(config: Config) -> Self {
        Self {
            state: Arc::new(RwLock::new(State::new(config))),
            state_watcher: None,
        }
    }

    pub fn start(&mut self) {
        if self.state_watcher.is_none() {
            let (tx, rx) = sync_channel::<Notification>(0);
            let watcher = Arc::new(StateWatcher { tx });
            self.state
                .write()
                .unwrap()
                .register(Arc::downgrade(&watcher) as Weak<dyn Observer + Send + Sync>);
            self.state_watcher = Some(watcher);
            let state = self.state.clone();
            std::thread::spawn(move || {
                loop {
                    let (res, next_run) = apply(&state);
                    state.write().unwrap().set_last_result(res);

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

    pub fn stop(&mut self) {
        self.state_watcher.as_ref().map(|s| {
            s.tx.send(Notification::Terminate).unwrap();
        });
        self.state_watcher = None;
    }

    #[allow(unused)]
    pub fn is_running(&self) -> bool {
        self.state_watcher.is_some()
    }
}

impl Drop for BrightnessController {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Communicates between the controller and the background thread
struct StateWatcher {
    tx: SyncSender<Notification>,
}

/// Update the background thread when the controller state changes
impl Observer for StateWatcher {
    fn did_set_enabled(&self, _: bool) {
        self.tx.send(Notification::Refresh).unwrap();
    }

    fn did_set_last_result(&self, _: &ApplyResult) {}

    fn did_set_config(&self, _: &Config) {
        self.tx.send(Notification::Refresh).unwrap();
    }
}
