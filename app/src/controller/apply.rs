use crate::brightness::calculate_brightness;
use crate::config::Config;
use brightness::{Brightness, BrightnessDevice};
use futures::{executor::block_on, StreamExt};
use std::collections::HashMap;
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

pub fn apply(config: &Config, enabled: bool) -> (ApplyResult, Option<unix_t>) {
    // Calculate sunrise and brightness
    match &config.location {
        None => {
            log::warn!("Unable to compute brightness because no location has been configured");
            (ApplyResult::Error(ApplyError::NoLocationSet), None)
        }
        Some(location) => {
            let now = SystemTime::now();
            let epoch_time_now = now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
            let input = sunrise_sunset_calculator::SscInput::new(
                epoch_time_now,
                location.latitude,
                location.longitude,
            );
            let ssr = input.compute().unwrap();
            let br = calculate_brightness(config, &ssr, epoch_time_now);
            log::info!("Computed base brightness of {}%", br.brightness);

            let results = SolarAndBrightnessResults {
                base_brightness: br.brightness,
                expiry: UNIX_EPOCH + Duration::from_secs(br.expiry_time as u64),
                time: now,
                sunrise: UNIX_EPOCH + Duration::from_secs(ssr.rise as u64),
                sunset: UNIX_EPOCH + Duration::from_secs(ssr.set as u64),
                visible: ssr.visible,
            };

            if enabled {
                let mut errors = vec![];
                let devices = block_on(get_devices());
                let devices_len = devices.len();
                for dev in devices {
                    match dev {
                        Ok(mut dev) => {
                            if let Err(e) = block_on(dev.set(br.brightness)) {
                                log::error!(
                                    "An error occurred setting monitor brightness: {:?} for: {:?}",
                                    e,
                                    dev
                                );
                                errors.push(e);
                            }
                        }
                        Err(e) => {
                            log::error!("An error occurred getting monitors: {:?}", e);
                            errors.push(e);
                        }
                    }
                }
                if errors.is_empty() {
                    log::info!(
                        "Brightness applied successfully to {} monitors",
                        devices_len
                    );
                }
                (
                    ApplyResult::Applied(results, Arc::new(errors)),
                    Some(br.expiry_time),
                )
            } else {
                log::info!("Dynamic brightness is disabled, skipping apply");
                (ApplyResult::Skipped(results), Some(br.expiry_time))
            }
        }
    }
}

pub async fn get_devices() -> Vec<Result<BrightnessDevice, brightness::Error>> {
    brightness::brightness_devices().collect::<Vec<_>>().await
}

#[cfg(windows)]
pub async fn get_properties(
    device: &BrightnessDevice,
) -> Result<HashMap<&'static str, String>, brightness::Error> {
    use brightness::BrightnessExt;
    Ok(hashmap! {
        "device_name" => device.device_name().await?,
        "device_description" => device.device_description().await?,
        "device_key" => device.device_registry_key().await?,
        "device_path" => device.device_path().await?,
    })
}

#[cfg(target_os = "linux")]
pub async fn get_properties(
    device: &BrightnessDevice,
) -> Result<HashMap<&'static str, String>, brightness::Error> {
    Ok(hashmap! {
        "device_name" => device.device_name().await?,
    })
}
