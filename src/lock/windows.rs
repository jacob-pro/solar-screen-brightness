use crate::wide::{set_and_get_error, WideString};
use anyhow::anyhow;
use solar_screen_brightness_windows_bindings::windows::HRESULT;
use solar_screen_brightness_windows_bindings::Windows::Win32::{
    Foundation::{BOOL, HANDLE, PWSTR},
    Security::{GetTokenInformation, TokenStatistics, TOKEN_QUERY, TOKEN_STATISTICS},
    System::Diagnostics::Debug::ERROR_ALREADY_EXISTS,
    System::Threading::{CreateMutexW, GetCurrentProcess, OpenProcessToken},
};
use std::ffi::c_void;

pub(super) struct Lock();

impl Lock {
    pub fn acquire() -> Option<Self> {
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
            let mut name_wide = name.as_str().to_wide();
            match set_and_get_error(|| {
                CreateMutexW(
                    std::ptr::null_mut(),
                    BOOL::from(true),
                    PWSTR(name_wide.as_mut_ptr()),
                )
            }) {
                Ok(_) => {
                    log::info!("Acquired lock: {}", name);
                    Some(Lock())
                }
                Err(e) if e.code() == HRESULT::from_win32(ERROR_ALREADY_EXISTS.0) => None,
                Err(e) => {
                    log::warn!(
                        "Unexpected error acquiring lock: {}, ignoring with dummy lock",
                        e
                    );
                    Some(Lock())
                }
            }
        }
    }
}
