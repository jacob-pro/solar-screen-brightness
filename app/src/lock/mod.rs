/// Make sure there is only one instance of solar screen brightness per session
#[cfg(unix)]
#[path = "unix.rs"]
mod lock_impl;

#[cfg(windows)]
#[path = "windows.rs"]
mod lock_impl;

#[cfg(unix)]
pub use lock_impl::ShowConsoleWatcher;

pub struct ApplicationLock(lock_impl::Lock);

pub struct ExistingProcess(lock_impl::Existing);

pub fn acquire() -> Result<ApplicationLock, ExistingProcess> {
    lock_impl::acquire()
        .map(|l| ApplicationLock(l))
        .map_err(|e| ExistingProcess(e))
}

impl ExistingProcess {
    #[inline]
    pub fn show_console_in_owning_process(&self) {
        log::info!("Attempting to show the already running application");
        if let Err(e) = self.0.show_console() {
            log::error!("Failed to show running application: {:#}", e);
        }
    }
}
