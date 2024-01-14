use crate::calculator::calculate_brightness;
use crate::config::{BrightnessValues, Location, MonitorOverride, MonitorProperty};
use brightness::blocking::{Brightness, BrightnessDevice};
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use sunrise_sunset_calculator::SunriseSunsetParameters;
use wildmatch::WildMatch;

#[derive(Debug, Serialize)]
pub struct ApplyResults {
    pub unknown_devices: Vec<String>,
    pub monitors: Vec<MonitorResult>,
    pub sun: SunriseSunsetResult,
}

#[derive(Debug, Serialize)]
pub struct SunriseSunsetResult {
    pub set: i64,
    pub rise: i64,
    pub visible: bool,
}

impl From<sunrise_sunset_calculator::SunriseSunsetResult> for SunriseSunsetResult {
    fn from(value: sunrise_sunset_calculator::SunriseSunsetResult) -> Self {
        Self {
            set: value.set,
            rise: value.rise,
            visible: value.visible,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MonitorResult {
    pub properties: MonitorProperties,
    pub brightness: Option<BrightnessDetails>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MonitorProperties {
    pub device_name: String,
    #[cfg(windows)]
    pub device_description: String,
    #[cfg(windows)]
    pub device_key: String,
    #[cfg(windows)]
    pub device_path: String,
}

#[derive(Debug, Serialize)]
pub struct BrightnessDetails {
    pub expiry_time: Option<i64>,
    pub brightness: u32,
    pub brightness_day: u32,
    pub brightness_night: u32,
}

pub struct MonitorOverrideCompiled {
    pub pattern: WildMatch,
    pub key: MonitorProperty,
    pub brightness: Option<BrightnessValues>,
}

impl From<&MonitorOverride> for MonitorOverrideCompiled {
    fn from(value: &MonitorOverride) -> Self {
        Self {
            pattern: WildMatch::new(&value.pattern),
            key: value.key,
            brightness: value.brightness.clone(),
        }
    }
}

/// Find the first override that matches this monitor's properties
fn match_monitor<'o>(
    overrides: &'o [MonitorOverrideCompiled],
    monitor: &MonitorProperties,
) -> Option<&'o MonitorOverrideCompiled> {
    let map = monitor.to_map();
    for o in overrides {
        if let Some(value) = map.get(&o.key) {
            if o.pattern.matches(value) {
                return Some(o);
            }
        }
    }
    None
}

pub fn apply_brightness(
    brightness_day: u32,
    brightness_night: u32,
    transition_mins: u32,
    location: Location,
    overrides: Vec<MonitorOverride>,
) -> ApplyResults {
    let overrides = overrides
        .iter()
        .map(MonitorOverrideCompiled::from)
        .collect::<Vec<_>>();
    let now = SystemTime::now();
    let epoch_time_now = now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let sun = SunriseSunsetParameters::new(epoch_time_now, location.latitude, location.longitude)
        .calculate()
        .unwrap();
    log::debug!("Now: {}, Sun: {:?}", epoch_time_now, sun);

    let mut failed_monitors = vec![];
    let mut monitors = vec![];
    brightness::blocking::brightness_devices().for_each(|m| match m {
        Ok(v) => monitors.push(v),
        Err(e) => failed_monitors.push(e),
    });
    log::debug!("Monitors: {:?}, Errors: {:?}", monitors, failed_monitors);

    let monitor_results = monitors
        .into_iter()
        .map(|m| {
            let properties = MonitorProperties::from_device(&m);
            let monitor_values = match match_monitor(&overrides, &properties) {
                None => Some(BrightnessValues {
                    brightness_day,
                    brightness_night,
                }),
                Some(o) => o.brightness.clone(),
            };

            if let Some(BrightnessValues {
                brightness_day,
                brightness_night,
            }) = monitor_values
            {
                let brightness = calculate_brightness(
                    brightness_day,
                    brightness_night,
                    transition_mins,
                    &sun,
                    epoch_time_now,
                );
                log::debug!(
                    "Computed brightness for '{}' = {:?} (day={}) (night={})",
                    properties.device_name,
                    brightness,
                    brightness_day,
                    brightness_night
                );

                let error = m.set(brightness.brightness).err();
                if let Some(err) = error.as_ref() {
                    log::error!(
                        "Failed to set brightness for '{}': {:?}",
                        properties.device_name,
                        err
                    );
                } else {
                    log::info!(
                        "Successfully set brightness for '{}' to {}%",
                        properties.device_name,
                        brightness.brightness
                    );
                }

                MonitorResult {
                    properties,
                    brightness: Some(BrightnessDetails {
                        expiry_time: brightness.expiry_time,
                        brightness: brightness.brightness,
                        brightness_day,
                        brightness_night,
                    }),
                    error: error.map(|e| e.to_string()),
                }
            } else {
                log::info!(
                    "Skipping '{}' due to monitor override",
                    properties.device_name
                );
                MonitorResult {
                    properties,
                    brightness: None,
                    error: None,
                }
            }
        })
        .sorted_by_key(|m| m.properties.device_name.clone())
        .collect::<Vec<_>>();

    ApplyResults {
        unknown_devices: failed_monitors.into_iter().map(|f| f.to_string()).collect(),
        monitors: monitor_results,
        sun: sun.into(),
    }
}

impl MonitorProperties {
    fn from_device(device: &BrightnessDevice) -> Self {
        #[cfg(windows)]
        use brightness::blocking::windows::BrightnessExt;
        Self {
            device_name: device.device_name().unwrap(),
            #[cfg(windows)]
            device_description: device.device_description().unwrap(),
            #[cfg(windows)]
            device_key: device.device_registry_key().unwrap(),
            #[cfg(windows)]
            device_path: device.device_path().unwrap(),
        }
    }

    pub fn to_map(&self) -> HashMap<MonitorProperty, &str> {
        let mut map = HashMap::<_, &str>::new();
        map.insert(MonitorProperty::DeviceName, &self.device_name);
        #[cfg(windows)]
        {
            map.insert(MonitorProperty::DeviceDescription, &self.device_description);
            map.insert(MonitorProperty::DeviceKey, &self.device_key);
            map.insert(MonitorProperty::DevicePath, &self.device_path);
        }
        map
    }
}
