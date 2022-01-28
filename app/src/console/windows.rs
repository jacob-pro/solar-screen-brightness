use crate::controller::BrightnessController;
use crate::tray::TrayApplicationHandle;
use crate::tui::launch_cursive;
use crate::APP_NAME;
use solar_screen_brightness_windows::{set_and_get_error, WindowDataExtension};
use std::sync::Arc;
use std::time::SystemTime;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        BringWindowToTop, CallWindowProcW, GetWindowLongPtrW, SetForegroundWindow,
        SetWindowLongPtrW, SetWindowTextW, ShowWindow, GWLP_USERDATA, GWL_WNDPROC, SC_CLOSE,
        SW_HIDE, SW_RESTORE, WM_CLOSE, WM_SYSCOMMAND, WNDPROC,
    },
};

// Passed as a pointer - it must be at a fixed heap address
struct WindowData {
    handle: HWND,
    old_proc: isize,
}

pub(super) struct Console {
    tray: TrayApplicationHandle,
    controller: Arc<BrightnessController>,
    window_data: Option<Box<WindowData>>,
}

impl Console {
    pub(super) fn new(tray: TrayApplicationHandle, controller: Arc<BrightnessController>) -> Self {
        Self {
            tray,
            controller,
            window_data: None,
        }
    }

    pub(super) fn show(&mut self) {
        if self.window_data.is_none() {
            self.initialise();
        }
        self.window_data.as_mut().unwrap().show();
    }

    pub(super) fn hide(&self) {
        if let Some(d) = self.window_data.as_ref() {
            d.hide()
        };
    }

    fn initialise(&mut self) {
        let tray = self.tray.clone();
        let controller = Arc::clone(&self.controller);
        launch_cursive(tray, controller);
        let handle = await_handle();
        let mut data = unsafe {
            Box::new(WindowData {
                handle,
                old_proc: GetWindowLongPtrW(handle, GWL_WNDPROC),
            })
        };
        unsafe {
            set_and_get_error(|| {
                SetWindowLongPtrW(data.handle, GWLP_USERDATA, data.as_mut() as *mut _ as isize)
            })
            .unwrap();
            set_and_get_error(|| {
                SetWindowLongPtrW(data.handle, GWL_WNDPROC, (window_proc as usize) as isize)
            })
            .unwrap();

            set_and_get_error(|| SetWindowTextW(data.handle, APP_NAME)).unwrap();
        }
        self.window_data = Some(data);
    }
}

impl WindowData {
    pub fn show(&self) {
        unsafe {
            set_and_get_error(|| ShowWindow(self.handle, SW_RESTORE)).unwrap();
            set_and_get_error(|| BringWindowToTop(self.handle)).unwrap();
            set_and_get_error(|| SetForegroundWindow(self.handle)).unwrap();
        }
    }

    pub fn hide(&self) {
        unsafe {
            set_and_get_error(|| ShowWindow(self.handle, SW_HIDE)).unwrap();
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let window_data: &mut WindowData = hwnd.get_user_data().unwrap();
    let intercept_close = || {
        log::info!("Intercepted console window close, hiding instead");
        window_data.hide();
        0
    };
    match msg {
        WM_CLOSE => {
            return intercept_close();
        }
        WM_SYSCOMMAND => {
            if wparam == (SC_CLOSE as usize) {
                return intercept_close();
            }
        }
        _ => {}
    };
    let ptr = window_data.old_proc as *mut ();
    let code: WNDPROC = std::mem::transmute(ptr);
    CallWindowProcW(code, hwnd, msg, wparam, lparam)
}

fn await_handle() -> HWND {
    extern "C" {
        // https://github.com/Bill-Gray/PDCursesMod/blob/44e38bbbdad146144d91fb6c28536975c27be54e/wingui/pdcscrn.c#L105
        // https://github.com/Bill-Gray/PDCursesMod/blob/master/wingui/pdcscrn.c#L2097
        static PDC_hWnd: HWND;
    }
    let start = SystemTime::now();
    loop {
        unsafe {
            let dur = SystemTime::now().duration_since(start).unwrap();
            let ms = dur.as_micros() as f64 / 1000.0;
            if !HANDLE(PDC_hWnd).is_invalid() {
                log::info!("Found valid PDC_hWnd in {:.2} ms", ms);
                return PDC_hWnd;
            } else if ms > 50.0 {
                log::warn!("Have not yet found a valid PDC_hWnd after {:.2} ms", ms);
            }
        }
    }
}
