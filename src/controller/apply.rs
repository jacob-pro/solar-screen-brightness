use crate::brightness::calculate_brightness;
use crate::controller::StateRef;
use crate::monitor::load_monitors;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use sunrise_sunset_calculator::binding::unix_t;

#[derive(Clone, Debug)]
pub struct BrightnessResults {
    pub base_brightness: u32,
    pub expiry: SystemTime,
    pub time: SystemTime,
    pub sunrise: SystemTime,
    pub sunset: SystemTime,
    pub visible: bool,
}

#[derive(Error, Debug, Clone)]
pub enum ApplyError {
    #[error("A location has not yet been set in the configuration")]
    NoLocationSet,
}

#[derive(Clone, Debug)]
pub enum ApplyResult {
    Skipped(BrightnessResults),
    Applied(BrightnessResults),
    Error(ApplyError),
}

pub fn apply(state: &StateRef) -> (ApplyResult, unix_t) {
    // Clone the latest config and apply it
    let config = state.read().unwrap().get_config().clone();
    // Calculate sunrise and brightness
    match &config.location {
        None => return (ApplyResult::Error(ApplyError::NoLocationSet), unix_t::MAX),
        Some(location) => {
            let now = SystemTime::now();
            let epoch_time_now = now
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let input = sunrise_sunset_calculator::SscInput::new(
                epoch_time_now,
                location.latitude,
                location.longitude,
            );
            let ssr = input.compute().unwrap();
            let br = calculate_brightness(&config, &ssr, epoch_time_now);

            if state.read().unwrap().get_enabled() {
                for m in load_monitors() {
                    m.set_brightness(br.brightness);
                }
            }

            let results = BrightnessResults {
                base_brightness: br.brightness,
                expiry: UNIX_EPOCH + Duration::from_secs(br.expiry_time as u64),
                time: now,
                sunrise: UNIX_EPOCH + Duration::from_secs(ssr.rise as u64),
                sunset: UNIX_EPOCH + Duration::from_secs(ssr.set as u64),
                visible: ssr.visible,
            };
            (ApplyResult::Applied(results), br.expiry_time)
        }
    }
}
