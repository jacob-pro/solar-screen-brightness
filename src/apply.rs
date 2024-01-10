use crate::calculator::calculate_brightness;
use crate::config::{BrightnessValues, Location, MonitorOverride};
use brightness::blocking::{Brightness, BrightnessDevice};
use maplit::btreemap;
use serde::Serialize;
use std::collections::BTreeMap;
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
    pub device_name: String,
    pub properties: BTreeMap<&'static str, String>,
    pub brightness: Option<BrightnessDetails>,
    pub error: Option<String>,
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
    pub key: String,
    pub brightness: Option<BrightnessValues>,
}

impl From<&MonitorOverride> for MonitorOverrideCompiled {
    fn from(value: &MonitorOverride) -> Self {
        Self {
            pattern: WildMatch::new(&value.pattern),
            key: value.key.clone(),
            brightness: value.brightness.clone(),
        }
    }
}

/// Find the first override that matches this monitor's properties
fn match_monitor<'o>(
    overrides: &'o [MonitorOverrideCompiled],
    monitor: &BTreeMap<&'static str, String>,
) -> Option<&'o MonitorOverrideCompiled> {
    for o in overrides {
        if let Some(value) = monitor.get(o.key.as_str()) {
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
            let device_name = m.device_name().unwrap_or_default();
            let properties = get_properties(&m).unwrap_or_default();
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
                    device_name,
                    brightness,
                    brightness_day,
                    brightness_night
                );

                let error = m.set(brightness.brightness).err();
                if let Some(err) = error.as_ref() {
                    log::error!("Failed to set brightness for '{}': {:?}", device_name, err);
                } else {
                    log::info!(
                        "Successfully set brightness for '{}' to {}%",
                        device_name,
                        brightness.brightness
                    );
                }

                MonitorResult {
                    device_name,
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
                log::info!("Skipping '{}' due to monitor override", device_name,);
                MonitorResult {
                    device_name,
                    properties,
                    brightness: None,
                    error: None,
                }
            }
        })
        .collect::<Vec<_>>();

    ApplyResults {
        unknown_devices: failed_monitors.into_iter().map(|f| f.to_string()).collect(),
        monitors: monitor_results,
        sun: sun.into(),
    }
}

#[cfg(windows)]
pub fn get_properties(
    device: &BrightnessDevice,
) -> Result<BTreeMap<&'static str, String>, brightness::Error> {
    use brightness::blocking::windows::BrightnessExt;
    Ok(btreemap! {
        "device_name" => device.device_name()?,
        "device_description" => device.device_description()?,
        "device_key" => device.device_registry_key()?,
        "device_path" => device.device_path()?,
    })
}

#[cfg(target_os = "linux")]
pub fn get_properties(
    device: &BrightnessDevice,
) -> Result<BTreeMap<&'static str, String>, brightness::Error> {
    Ok(btreemap! {
        "device_name" => device.device_name()?,
    })
}
