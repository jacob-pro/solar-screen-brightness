use crate::config::Config;
use std::sync::mpsc::{SyncSender, sync_channel, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crate::ssc::{ssc_around_time, SSCAroundTimeResult, ssc_calculate_brightness, SSCBrightnessParams, SSCStatus_SSCStatusSuccess};
use std::sync::{Arc, RwLock, Weak};
use crate::monitor::load_monitors;

pub type BrightnessMessageSender = SyncSender<BrightnessMessage>;
pub type BrightnessStatusRef = Arc<RwLock<BrightnessStatus>>;

pub trait BrightnessStatusDelegate {
    fn running_change(&self, running: &bool);
    fn update_change(&self, update: &LastCalculation);
}

#[derive(Clone)]
pub struct LastCalculation {
    pub brightness: u32,
    pub expiry: SystemTime,
    pub time: SystemTime,
    pub sunrise: SystemTime,
    pub sunset: SystemTime,
    pub visible: bool,
}

pub struct BrightnessStatus {
    last_calculation: Option<LastCalculation>,
    enabled: bool,
    pub config: Config,
    pub delegate: Weak<Box<dyn BrightnessStatusDelegate + Send + Sync>>,
}

impl BrightnessStatus {
    pub fn last_calculation(&self) -> &Option<LastCalculation> { &self.last_calculation }
    pub fn is_enabled(&self) -> bool { self.enabled }

    fn set_enabled(&mut self, running: bool) {
        self.delegate.upgrade().map(|x| x.running_change(&running));
        self.enabled = running;
    }

    fn set_last_calculation(&mut self, update: LastCalculation) {
        self.delegate.upgrade().map(|x| x.update_change(&update));
        self.last_calculation = Some(update);
    }
}

pub enum BrightnessMessage {
    NewConfig,
    Exit,
    Disable,
    Enable,
}

// Launches brightness on background thread
pub fn run(config: Config) -> (BrightnessMessageSender, BrightnessStatusRef) {
    let (tx, rx) = sync_channel::<BrightnessMessage>(0);
    let status2 = Arc::new(RwLock::new(BrightnessStatus {
        last_calculation: None,
        config: config.clone(),
        enabled: true,
        delegate: Weak::new()
    }));
    let status = status2.clone();
    thread::spawn(move || {
        loop {
            // Load the latest config
            let config = status.read().unwrap().config.clone();
            // Calculate sunrise and brightness
            let now = SystemTime::now();
            let (ssr, br) = unsafe {
                let mut sunrise_sunset_result: SSCAroundTimeResult = std::mem::MaybeUninit::zeroed().assume_init();
                let status = ssc_around_time(config.location.latitude.into(),
                                config.location.longitude.into(),
                                now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                                &mut sunrise_sunset_result);
                assert_eq!(status, SSCStatus_SSCStatusSuccess);
                let params = SSCBrightnessParams {
                    brightness_day: config.brightness_day,
                    brightness_night: config.brightness_night,
                    transition_mins: config.transition_mins,
                };
                let brightness_result = ssc_calculate_brightness(&params, &sunrise_sunset_result);
                (sunrise_sunset_result, brightness_result)
            };
            let update_start = Instant::now();

            if status.read().unwrap().enabled {

                for m in load_monitors() {
                    m.set_brightness(br.brightness);
                }

            }

            let mut status_w = status.write().unwrap();
            status_w.config = config.clone();
            status_w.set_last_calculation(LastCalculation {
                brightness: br.brightness,
                expiry: now + Duration::new(br.expiry_seconds as u64, 0),
                time: now,
                sunrise: UNIX_EPOCH + Duration::from_secs(ssr.rise as u64),
                sunset: UNIX_EPOCH + Duration::from_secs(ssr.set as u64),
                visible: ssr.visible
            });
            drop(status_w);

            match rx.recv_timeout(Duration::from_secs(br.expiry_seconds as u64 - update_start.elapsed().as_secs())) {
                Ok(msg) => {
                    match msg {
                        BrightnessMessage::NewConfig => {}
                        BrightnessMessage::Exit => { break }
                        BrightnessMessage::Disable => {
                            status.write().unwrap().set_enabled(false);
                        }
                        BrightnessMessage::Enable => {
                            status.write().unwrap().set_enabled(true);
                        }
                    }
                }
                Err(e) => { if e != RecvTimeoutError::Timeout { panic!(e)}}
            };
        }
    });
    (tx, status2)
}

