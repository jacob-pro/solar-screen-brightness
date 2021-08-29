use std::ffi::c_void;

/// Make sure there is only one instance of solar screen brightness per session

#[cfg(target_os = "windows")]
pub fn acquire_lock() -> bool {
    use crate::wide::WideString;
    use solar_screen_brightness_windows_bindings::Windows::Win32::{
        Foundation::{BOOL, HANDLE, PWSTR},
        Security::{GetTokenInformation, TokenStatistics, TOKEN_QUERY, TOKEN_STATISTICS},
        System::Diagnostics::Debug::{GetLastError, SetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::{CreateMutexW, GetCurrentProcess, OpenProcessToken},
    };
    unsafe {
        // https://www.codeproject.com/Articles/538/Avoiding-Multiple-Instances-of-an-Application
        let mut name = String::from("solar-screen-brightness");
        let mut token = HANDLE::NULL;
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).as_bool() {
            let mut len = 0;
            let mut data: TOKEN_STATISTICS = std::mem::MaybeUninit::zeroed().assume_init();
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
        SetLastError(0);
        if CreateMutexW(
            std::ptr::null_mut(),
            BOOL::from(true),
            PWSTR(name_wide.as_mut_ptr()),
        )
        .is_invalid()
        {
            let e = GetLastError();
            if e == ERROR_ALREADY_EXISTS {
                log::error!("Failed to acquire lock - the application is already running");
                return false;
            } else {
                log::warn!(
                    "Failed to acquire lock, system error code: {}, ignoring",
                    e.0
                )
            }
        } else {
            log::info!("Acquired lock: {}", name);
        }
    }
    true
}

#[cfg(not(target_os = "windows"))]
pub fn acquire_lock() -> bool {
    true
}
