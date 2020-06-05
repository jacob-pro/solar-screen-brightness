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
    fn update_change(&self, update: &LastUpdate);
}

#[derive(Clone)]
pub struct LastUpdate {
    brightness: u32,
    expiry: SystemTime,
    time: SystemTime,
    sunrise: SystemTime,
    sunset: SystemTime,
    visible: bool,
}

pub struct BrightnessStatus {
    last_update: Option<LastUpdate>,
    config: Config,
    running: bool,
    pub delegate: Weak<Box<dyn BrightnessStatusDelegate + Send + Sync>>,
}

impl BrightnessStatus {
    pub fn last_update(&self) -> &Option<LastUpdate> { &self.last_update }
    pub fn config(&self) -> &Config { &self.config }
    pub fn running(&self) -> &bool { &self.running }

    fn set_running(&mut self, running: bool) {
        self.delegate.upgrade().map(|x| x.running_change(&running));
        self.running = running;
    }

    fn set_last_update(&mut self, update: LastUpdate) {
        self.delegate.upgrade().map(|x| x.update_change(&update));
        self.last_update = Some(update);
    }
}

pub enum BrightnessMessage {
    NewConfig(Config),
    Exit,
    Pause,
    Resume,
}

// Launches brightness on background thread
pub fn run(config: Config) -> (BrightnessMessageSender, BrightnessStatusRef) {
    let (tx, rx) = sync_channel::<BrightnessMessage>(0);
    let status2 = Arc::new(RwLock::new(BrightnessStatus {
        last_update: None,
        config: config.clone(),
        running: true,
        delegate: Weak::new()
    }));
    let status = status2.clone();
    thread::spawn(move || {
        let mut config = config;
        loop {
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

            for m in load_monitors() {
                m.set_brightness(br.brightness);
            }

            let mut status_w = status.write().unwrap();
            status_w.config = config.clone();
            status_w.set_last_update(LastUpdate {
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
                        BrightnessMessage::NewConfig(new_config) => {config = new_config}
                        BrightnessMessage::Exit => { break }
                        BrightnessMessage::Pause => {
                            status.write().unwrap().set_running(false);
                            loop {
                                match rx.recv().unwrap() {
                                    BrightnessMessage::Resume => {
                                        status.write().unwrap().set_running(true);
                                        break
                                    }
                                    _ => {}  // Ignore repeat Pause messages
                                }
                            }
                        }
                        BrightnessMessage::Resume => {}
                    }
                }
                Err(e) => { if e != RecvTimeoutError::Timeout { panic!(e)}}
            };
        }
    });
    (tx, status2)
}

