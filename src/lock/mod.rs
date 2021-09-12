/// Make sure there is only one instance of solar screen brightness per session
#[cfg(unix)]
#[path = "unix.rs"]
mod lock_impl;

#[cfg(windows)]
#[path = "windows.rs"]
mod lock_impl;

pub struct ApplicationLock(lock_impl::Lock);

impl ApplicationLock {
    #[inline]
    pub fn acquire() -> Option<Self> {
        lock_impl::Lock::acquire().map(|l| ApplicationLock(l))
    }

    #[cfg(unix)]
    #[inline]
    pub fn should_show_console(&self) -> bool {
        self.0.should_show_console()
    }

    #[inline]
    pub fn show_console_in_owning_process() {
        log::info!("Attempting to show the already running application");
        if let Err(e) = lock_impl::Lock::show_console_in_owning_process() {
            log::error!("Failed to show running application: {:#}", e);
        }
    }
}
