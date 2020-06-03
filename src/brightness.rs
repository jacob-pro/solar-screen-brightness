use crate::config::Config;
use std::sync::mpsc::{SyncSender, sync_channel, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};
use crate::ssc::{ssc_around_time, SSCAroundTimeResult, ssc_calculate_brightness, SSCBrightnessParams};
use libc::time;
use std::sync::{Arc, RwLock, Weak};
use crate::monitor::load_monitors;

pub type BrightnessMessageSender = SyncSender<BrightnessMessage>;
pub type BrightnessStatusRef = Arc<RwLock<BrightnessStatus>>;

pub trait BrightnessStatusDelegate {
    fn on_toggle(&self, running: bool);
}

pub struct BrightnessStatus {
    brightness: Option<u32>,
    expiry: Option<Instant>,
    config: Config,
    running: bool,
    pub delegate: Weak<Box<dyn BrightnessStatusDelegate + Send + Sync>>,
}

impl BrightnessStatus {
    pub fn brightness(&self) -> &Option<u32> { &self.brightness }
    pub fn expiry(&self) -> &Option<Instant> { &self.expiry }
    pub fn config(&self) -> &Config { &self.config }
    pub fn running(&self) -> &bool { &self.running }
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
        brightness: None,
        expiry: None,
        config: config.clone(),
        running: true,
        delegate: Weak::new()
    }));
    let status = status2.clone();
    thread::spawn(move || {
        let mut config = config;
        loop {

            let brightness_result = unsafe {
                let mut sunrise_sunset_result: SSCAroundTimeResult = std::mem::MaybeUninit::zeroed().assume_init();
                ssc_around_time(config.location.latitude.into(),
                                config.location.longitude.into(),
                                time(std::ptr::null_mut()),
                                &mut sunrise_sunset_result);
                let params = SSCBrightnessParams {
                    brightness_day: config.brightness_day,
                    brightness_night: config.brightness_night,
                    transition_mins: config.transition_mins,
                };
                ssc_calculate_brightness(&params, &sunrise_sunset_result)
            };
            let expiry = Instant::now() + Duration::new(brightness_result.expiry_seconds as u64, 0);

            // Update brightness
            for m in load_monitors() {
                m.set_brightness(brightness_result.brightness);
            }

            // Update status
            let mut status_w = status.write().unwrap();
            status_w.config = config.clone();
            status_w.brightness = Some(brightness_result.brightness);
            status_w.expiry = Some(expiry);
            drop(status_w);

            match rx.recv_timeout(expiry - Instant::now()) {
                Ok(msg) => {
                    match msg {
                        BrightnessMessage::NewConfig(new_config) => {config = new_config}
                        BrightnessMessage::Exit => { break }
                        BrightnessMessage::Pause => {
                            let mut status_w = status.write().unwrap();
                            status_w.running = false;
                            status_w.delegate.upgrade().map(|x| x.on_toggle(false));
                            drop(status_w);
                            loop {
                                match rx.recv().unwrap() {
                                    BrightnessMessage::Resume => {
                                        let mut status_w = status.write().unwrap();
                                        status_w.running = true;
                                        status_w.delegate.upgrade().map(|x| x.on_toggle(true));
                                        drop(status_w);
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

