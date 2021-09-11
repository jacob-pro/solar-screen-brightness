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
    pub fn should_show_console(&self) -> bool {
        self.0.should_show_console()
    }
}
