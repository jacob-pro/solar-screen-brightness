use solar_screen_brightness_windows::set_and_get_error;
use solar_screen_brightness_windows::windows::HRESULT;
use solar_screen_brightness_windows::Windows::Win32::{
    Foundation::{BOOL, HANDLE},
    Security::{GetTokenInformation, TokenStatistics, TOKEN_QUERY, TOKEN_STATISTICS},
    System::Diagnostics::Debug::ERROR_ALREADY_EXISTS,
    System::Threading::{CreateMutexW, GetCurrentProcess, OpenProcessToken},
};
use std::ffi::c_void;

pub struct Lock();

pub fn acquire() -> Result<Lock, Existing> {
    unsafe {
        // https://www.codeproject.com/Articles/538/Avoiding-Multiple-Instances-of-an-Application
        let mut name = String::from("solar-screen-brightness");
        let mut token = HANDLE::NULL;
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).as_bool() {
            let mut len = 0;
            let mut data = TOKEN_STATISTICS::default();
            let ptr = ((&mut data) as *mut TOKEN_STATISTICS) as *mut c_void;
            if GetTokenInformation(
                token,
                TokenStatistics,
                ptr,
                std::mem::size_of_val(&data) as u32,
                &mut len,
            )
            .as_bool()
            {
                let luid = data.AuthenticationId;
                name.push_str(format!("-{:x}{:x}", luid.HighPart, luid.LowPart).as_str())
            } else {
                log::warn!("GetTokenInformation failed when generating lock name");
            };
        }
        match set_and_get_error(|| {
            CreateMutexW(std::ptr::null_mut(), BOOL::from(true), name.as_str())
        }) {
            Ok(_) => {
                log::info!("Acquired lock: {}", name);
                Ok(Lock())
            }
            Err(e) if e.code() == HRESULT::from_win32(ERROR_ALREADY_EXISTS.0) => Err(Existing()),
            Err(e) => {
                log::warn!(
                    "Unexpected error acquiring lock: {}, ignoring with dummy lock",
                    e
                );
                Ok(Lock())
            }
        }
    }
}

pub struct Existing();

impl Existing {
    pub fn show_console(&self) -> Result<(), anyhow::Error> {
        crate::tray::show_console_in_owning_process()
    }
}
