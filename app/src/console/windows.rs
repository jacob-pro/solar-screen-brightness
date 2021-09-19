use crate::assets::Assets;
use crate::controller::BrightnessController;
use crate::tray::TrayApplicationHandle;
use crate::tui::launch_cursive;
use crate::APP_NAME;
use solar_screen_brightness_windows::Windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        BringWindowToTop, CallWindowProcW, CreateIconFromResource, GetWindowLongPtrW, SendMessageW,
        SetForegroundWindow, SetWindowLongPtrW, SetWindowTextW, ShowWindow, GWLP_USERDATA,
        GWL_WNDPROC, ICON_BIG, ICON_SMALL, SC_CLOSE, SW_HIDE, SW_RESTORE, WM_CLOSE, WM_SETICON,
        WM_SYSCOMMAND, WNDPROC,
    },
};
use solar_screen_brightness_windows::{set_and_get_error, WindowDataExtension};
use std::time::SystemTime;

// Passed as a pointer - it must be at a fixed heap address
struct WindowData {
    handle: HWND,
    old_proc: isize,
}

pub(super) struct Console {
    tray: TrayApplicationHandle,
    controller: BrightnessController,
    window_data: Option<Box<WindowData>>,
}

impl Console {
    pub(super) fn new(tray: TrayApplicationHandle, controller: BrightnessController) -> Self {
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
        self.window_data.as_ref().map(|d| d.hide());
    }

    fn initialise(&mut self) {
        let tray = self.tray.clone();
        let controller = self.controller.clone();
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
            set_and_get_error(|| SetWindowLongPtrW(data.handle, GWL_WNDPROC, window_proc as isize))
                .unwrap();

            set_and_get_error(|| SetWindowTextW(data.handle, APP_NAME)).unwrap();
            let mut asset = Assets::get("icon-256.png")
                .expect("Icon missing")
                .into_owned();
            let hicon = set_and_get_error(|| {
                CreateIconFromResource(
                    asset.as_mut_ptr(),
                    asset.len() as u32,
                    BOOL::from(true),
                    0x00030000,
                )
            })
            .unwrap();
            set_and_get_error(|| {
                SendMessageW(
                    data.handle,
                    WM_SETICON,
                    WPARAM(ICON_BIG as usize),
                    LPARAM(hicon.0),
                )
            })
            .unwrap();
            set_and_get_error(|| {
                SendMessageW(
                    data.handle,
                    WM_SETICON,
                    WPARAM(ICON_SMALL as usize),
                    LPARAM(hicon.0),
                )
            })
            .unwrap();
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
        LRESULT(0)
    };
    match msg {
        WM_CLOSE => {
            return intercept_close();
        }
        WM_SYSCOMMAND => {
            if wparam == WPARAM(SC_CLOSE as usize) {
                return intercept_close();
            }
        }
        _ => {}
    };
    let ptr = window_data.old_proc as *mut ();
    let code: WNDPROC = std::mem::transmute(ptr);
    CallWindowProcW(Some(code), hwnd, msg, wparam, lparam)
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
            if !PDC_hWnd.is_null() {
                log::info!("Found valid PDC_hWnd in {:.2} ms", ms);
                return PDC_hWnd;
            } else {
                if ms > 50.0 {
                    log::warn!("Have not yet found a valid PDC_hWnd after {:.2} ms", ms);
                }
            }
        }
    }
}
