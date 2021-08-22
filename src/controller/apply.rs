use crate::brightness::calculate_brightness;
use crate::controller::StateRef;
use brightness::{Brightness, BrightnessDevice};
use futures::{executor::block_on, StreamExt};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sunrise_sunset_calculator::binding::unix_t;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct SolarAndBrightnessResults {
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
    Skipped(SolarAndBrightnessResults),
    Applied(SolarAndBrightnessResults, Arc<Vec<brightness::Error>>),
    Error(ApplyError),
    None,
}

pub fn apply(state: &StateRef) -> (ApplyResult, unix_t) {
    // Clone the latest config and apply it
    let config = state.read().unwrap().get_config().clone();
    // Calculate sunrise and brightness
    match &config.location {
        None => return (ApplyResult::Error(ApplyError::NoLocationSet), unix_t::MAX),
        Some(location) => {
            let now = SystemTime::now();
            let epoch_time_now = now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            let input = sunrise_sunset_calculator::SscInput::new(
                epoch_time_now,
                location.latitude,
                location.longitude,
            );
            let ssr = input.compute().unwrap();
            let br = calculate_brightness(&config, &ssr, epoch_time_now);

            let results = SolarAndBrightnessResults {
                base_brightness: br.brightness,
                expiry: UNIX_EPOCH + Duration::from_secs(br.expiry_time as u64),
                time: now,
                sunrise: UNIX_EPOCH + Duration::from_secs(ssr.rise as u64),
                sunset: UNIX_EPOCH + Duration::from_secs(ssr.set as u64),
                visible: ssr.visible,
            };

            if state.read().unwrap().get_enabled() {
                let mut errors = vec![];
                let devices = block_on(get_devices());
                for dev in devices {
                    match dev {
                        Ok(mut dev) => match block_on(dev.set(br.brightness)) {
                            Err(e) => {
                                errors.push(e);
                            }
                            _ => {}
                        },
                        Err(e) => {
                            errors.push(e);
                        }
                    }
                }
                (
                    ApplyResult::Applied(results, Arc::new(errors)),
                    br.expiry_time,
                )
            } else {
                (ApplyResult::Skipped(results), br.expiry_time)
            }
        }
    }
}

async fn get_devices() -> Vec<Result<BrightnessDevice, brightness::Error>> {
    brightness::brightness_devices().collect::<Vec<_>>().await
}
