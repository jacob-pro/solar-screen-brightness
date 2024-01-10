use crate::apply::{apply_brightness, ApplyResults};
use crate::config::SsbConfig;
use human_repr::HumanDuration;
use std::mem::take;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub enum Message {
    Shutdown,
    Refresh(&'static str),
    Disable(&'static str),
    Enable(&'static str),
}

pub struct BrightnessController {
    pub sender: mpsc::Sender<Message>,
    pub last_result: Arc<RwLock<Option<ApplyResults>>>,
    join_handle: Option<JoinHandle<()>>,
}

impl BrightnessController {
    pub fn start<F: Fn() + Send + 'static>(
        config: Arc<RwLock<SsbConfig>>,
        on_update: F,
    ) -> BrightnessController {
        let (sender, receiver) = mpsc::channel();
        let last_result = Arc::new(RwLock::new(None));
        let cloned = last_result.clone();
        let join_handle = thread::spawn(move || {
            run(config, receiver, cloned, on_update);
        });
        BrightnessController {
            sender,
            last_result,
            join_handle: Some(join_handle),
        }
    }
}

impl Drop for BrightnessController {
    fn drop(&mut self) {
        self.sender.send(Message::Shutdown).unwrap();
        take(&mut self.join_handle).unwrap().join().unwrap();
        log::debug!("Stopped BrightnessController");
    }
}

fn run<F: Fn()>(
    config: Arc<RwLock<SsbConfig>>,
    receiver: mpsc::Receiver<Message>,
    last_result: Arc<RwLock<Option<ApplyResults>>>,
    on_update: F,
) {
    log::info!("Starting BrightnessController");
    let mut enabled = true;

    loop {
        let timeout = if enabled {
            // Apply brightness using latest config
            let config = config.read().unwrap().clone();
            let result = apply(config);
            let timeout = calculate_timeout(&result);

            // Update last result
            *last_result.write().unwrap() = result;
            on_update();
            timeout
        } else {
            log::info!("BrightnessController is disabled, skipping update");
            None
        };

        // Sleep until receiving message or timeout
        let rx_result = match timeout {
            None => {
                log::info!("Brightness Worker sleeping indefinitely");
                receiver.recv().map_err(|e| e.into())
            }
            Some(timeout) => {
                let duration = timeout
                    .duration_since(SystemTime::now())
                    .unwrap_or_default();
                log::info!(
                    "BrightnessController sleeping for {}s",
                    duration.human_duration()
                );
                receiver.recv_timeout(duration)
            }
        };

        match rx_result {
            Ok(Message::Shutdown) => {
                log::info!("Stopping BrightnessController");
                break;
            }
            Ok(Message::Refresh(src)) => {
                log::info!("Refreshing due to '{src}'");
            }
            Ok(Message::Disable(src)) => {
                log::info!("Disabling BrightnessController due to '{src}'");
                enabled = false;
            }
            Ok(Message::Enable(src)) => {
                log::info!("Enabling BrightnessController due to '{src}'");
                enabled = true;
            }
            Err(RecvTimeoutError::Timeout) => {
                log::debug!("Refreshing due to timeout")
            }
            Err(RecvTimeoutError::Disconnected) => panic!("Unexpected disconnection"),
        }
    }
}

// The time at which the brightness should be re-applied
fn calculate_timeout(results: &Option<ApplyResults>) -> Option<SystemTime> {
    if let Some(results) = results {
        results
            .monitors
            .iter()
            .flat_map(|m| m.brightness.as_ref().map(|b| b.expiry_time))
            .flatten()
            .min()
            .map(|e| UNIX_EPOCH + Duration::from_secs(e as u64))
    } else {
        None
    }
}

// Calculate and apply the brightness
fn apply(config: SsbConfig) -> Option<ApplyResults> {
    if let Some(location) = config.location {
        Some(apply_brightness(
            config.brightness_day,
            config.brightness_night,
            config.transition_mins,
            location,
            config.overrides,
        ))
    } else {
        log::warn!("Skipping apply because no location is configured");
        None
    }
}
