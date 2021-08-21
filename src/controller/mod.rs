pub mod state;
pub mod run_result;

use crate::brightness::calculate_brightness;
use crate::config::Config;
use crate::controller::state::Observer;
use crate::monitor::load_monitors;
use state::{LastResult, State};
use std::sync::mpsc::{sync_channel, RecvTimeoutError, SyncSender};
use std::sync::{Arc, RwLock, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
            state_watcher: None
        }
    }

    pub fn start(&mut self) {
        if !self.state_watcher.is_none() {
            let (tx, rx) = sync_channel::<Notification>(0);
            let watcher = Arc::new(StateWatcher { tx });
            self.state.write().unwrap().register(Arc::downgrade(&watcher) as Weak<dyn Observer + Send + Sync>);
            self.state_watcher = Some(watcher);
            let state = self.state.clone();
            std::thread::spawn(move || {
                loop {
                    // Clone the latest config and apply it
                    let config = state.read().unwrap().get_config().clone();
                    // Calculate sunrise and brightness
                    let now = SystemTime::now();
                    let epoch_time_now = now
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    let input = sunrise_sunset_calculator::SscInput::new(
                        epoch_time_now,
                        config.location.latitude.into(),
                        config.location.longitude.into(),
                    );
                    let ssr = input.compute().unwrap();
                    let br = calculate_brightness(&config, &ssr, epoch_time_now);

                    let update_start = Instant::now();
                    if state.read().unwrap().get_enabled() {
                        for m in load_monitors() {
                            m.set_brightness(br.brightness);
                        }
                    }

                    // Write back the results of this run
                    let mut state_w = state.write().unwrap();
                    state_w.set_last_result(LastResult {
                        brightness: br.brightness,
                        expiry: now + Duration::new(br.expiry_seconds as u64, 0),
                        time: now,
                        sunrise: UNIX_EPOCH + Duration::from_secs(ssr.rise as u64),
                        sunset: UNIX_EPOCH + Duration::from_secs(ssr.set as u64),
                        visible: ssr.visible,
                    });
                    drop(state_w);

                    // Wait for the next run, or a notification
                    match rx.recv_timeout(Duration::from_secs(
                        br.expiry_seconds as u64 - update_start.elapsed().as_secs(),
                    )) {
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

    fn did_set_last_result(&self, _: &LastResult) {}

    fn did_set_config(&self, _: &Config) {
        self.tx.send(Notification::Refresh).unwrap();
    }
}
