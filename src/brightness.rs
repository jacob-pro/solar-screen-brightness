use crate::config::Config;
use std::sync::mpsc::{SyncSender, sync_channel, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};
use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, LPMONITORINFO, EnumDisplayDevicesW};
use winapi::shared::minwindef::{BOOL, LPARAM, TRUE, FALSE, DWORD};
use winapi::shared::windef::{LPRECT, LPCRECT, HDC, HMONITOR};
use winapi::shared::ntdef::NULL;
use winapi::um::physicalmonitorenumerationapi::{GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR, PHYSICAL_MONITOR};
use winapi::um::highlevelmonitorconfigurationapi::SetMonitorBrightness;
use winapi::um::winnt::LPCWSTR;
use winapi::um::wingdi::DISPLAY_DEVICEW;
use crate::wide::wide_to_str;
use crate::ssc::{ssc_around_time, SSCAroundTimeResult, ssc_calculate_brightness, SSCBrightnessParams};
use libc::time;
use std::sync::{Arc, RwLock, Weak};

pub type BrightnessMessageSender = SyncSender<BrightnessLoopMessage>;
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

pub enum BrightnessLoopMessage {
    NewConfig(Config),
    Exit,
    Pause,
    Resume,
}

// Launches brightness on background thread
pub fn run(config: Config) -> (BrightnessMessageSender, BrightnessStatusRef) {
    let (tx, rx) = sync_channel::<BrightnessLoopMessage>(0);
    let status = Arc::new(RwLock::new(BrightnessStatus {
        brightness: None,
        expiry: None,
        config: config.clone(),
        running: true,
        delegate: Weak::new()
    }));
    let status_mv = status.clone();
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
            let mut status = status_mv.write().unwrap();
            status.config = config.clone();
            status.brightness = Some(brightness_result.brightness);
            status.expiry = Some(expiry);
            drop(status);

            match rx.recv_timeout(expiry - Instant::now()) {
                Ok(msg) => {
                    match msg {
                        BrightnessLoopMessage::NewConfig(new_config) => {config = new_config}
                        BrightnessLoopMessage::Exit => { break }
                        BrightnessLoopMessage::Pause => {
                            let mut status = status_mv.write().unwrap();
                            status.running = false;
                            status.delegate.upgrade().map(|x| x.on_toggle(false));
                            drop(status);
                            loop {
                                match rx.recv().unwrap() {
                                    BrightnessLoopMessage::Resume => {
                                        let mut status = status_mv.write().unwrap();
                                        status.running = true;
                                        status.delegate.upgrade().map(|x| x.on_toggle(true));
                                        drop(status);
                                        break
                                    }
                                    _ => {}  // Ignore repeat Pause messages
                                }
                            }
                        }
                        BrightnessLoopMessage::Resume => {}
                    }
                }
                Err(e) => { if e != RecvTimeoutError::Timeout { panic!(e)}}
            };
        }
    });
    (tx, status)
}

pub fn load_monitors() -> Vec<Monitor> {
    unsafe extern "system" fn enum_monitors(arg1: HMONITOR, _arg2: HDC, _arg3: LPRECT, arg4: LPARAM) -> BOOL {
        let monitors: &mut Vec<HMONITOR> = &mut *(arg4 as *mut Vec<HMONITOR>);
        monitors.push(arg1);
        return TRUE
    }
    unsafe {
        let mut hmonitors = Vec::<HMONITOR>::new();
        if EnumDisplayMonitors(NULL as HDC,
                               NULL as LPCRECT,
                               Some(enum_monitors),
                               &mut hmonitors as *mut _ as isize
        ) == FALSE { panic!("EnumDisplayMonitors failed ")};
        hmonitors.into_iter().map(|x| Monitor::new(x)).collect()
    }
}

#[allow(dead_code)]
pub struct Monitor {
    handle: HMONITOR,
    physical_monitors: Vec<PHYSICAL_MONITOR>,
    device_name: String,
    device_string: String,
}

impl Monitor {

    unsafe fn new(handle: HMONITOR) -> Self {

        let mut count: DWORD = 0;
        if GetNumberOfPhysicalMonitorsFromHMONITOR(handle, &mut count) != TRUE {
            panic!("GetNumberOfPhysicalMonitorsFromHMONITOR failed")};
        let mut physical = Vec::with_capacity(count as usize);
        if GetPhysicalMonitorsFromHMONITOR(handle, count, physical.as_mut_ptr()) != TRUE {
            panic!("GetPhysicalMonitorsFromHMONITOR failed")};
        physical.set_len(count as usize);

        let mut info: MONITORINFOEXW = std::mem::MaybeUninit::zeroed().assume_init();
        info.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        let pointer = &mut info as *mut _;
        if GetMonitorInfoW(handle, pointer as LPMONITORINFO) == 0 { panic!("GetMonitorInfoW failed")};


        let mut device: DISPLAY_DEVICEW = std::mem::MaybeUninit::zeroed().assume_init();
        device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        if EnumDisplayDevicesW(&info.szDevice as LPCWSTR, 0, &mut device, 0) == 0 {
            panic!("EnumDisplayDevicesW failed")};

        Monitor {
            handle,
            physical_monitors: physical,
            device_name: wide_to_str(&info.szDevice).unwrap(),
            device_string: wide_to_str(&device.DeviceString).unwrap(),
        }
    }

    fn set_brightness(&self, brightness: u32) {
        unsafe {
            for p in &self.physical_monitors {
                SetMonitorBrightness(p.hPhysicalMonitor, brightness as DWORD);
            }
        }
    }

}

