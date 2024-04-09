use crate::unique::Error;
use win32_utils::instance::UniqueInstance;
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, HWND_BROADCAST};

const APP_ID: &str = "solar-screen-brightness";

#[allow(dead_code)]
pub struct SsbUniqueInstance(UniqueInstance);

impl SsbUniqueInstance {
    pub fn try_acquire() -> Result<Self, Error> {
        match UniqueInstance::acquire_unique_to_session(APP_ID) {
            Ok(u) => Ok(SsbUniqueInstance(u)),
            Err(win32_utils::instance::Error::AlreadyExists) => {
                Err(Error::AlreadyRunning(ExistingInstance))
            }
            Err(e) => Err(Error::PlatformError(Box::new(e))),
        }
    }
}

/// Represents an already running instance of Solar Screen Brightness
#[derive(Debug)]
pub struct ExistingInstance;

impl ExistingInstance {
    /// Sends a broadcast message to the existing instance, telling it to maximise its window
    pub fn wakeup(&self) {
        let message = crate::event_watcher::windows::register_open_window_message();
        unsafe { SendMessageW(HWND_BROADCAST, message, None, None) };
    }
}
