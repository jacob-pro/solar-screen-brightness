use crate::assets::Assets;
use crate::controller::BrightnessController;
use crate::tray::TrayApplicationHandle;
use crate::tui::run_cursive;
use crate::wide::{get_user_data, WideString};

use solar_screen_brightness_windows_bindings::Windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, PWSTR, WPARAM},
    UI::WindowsAndMessaging::{
        BringWindowToTop, CallWindowProcW, CreateIconFromResource, GetWindowLongPtrW, SendMessageW,
        SetForegroundWindow, SetWindowLongPtrW, SetWindowTextW, ShowWindow, GWLP_USERDATA,
        GWL_WNDPROC, ICON_BIG, ICON_SMALL, SC_CLOSE, SW_HIDE, SW_RESTORE, WM_CLOSE, WM_SETICON,
        WM_SYSCOMMAND, WNDPROC,
    },
};

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
        std::thread::spawn(move || {
            run_cursive(tray, controller);
        });
        let handle = await_handle();
        let mut data = unsafe {
            Box::new(WindowData {
                handle,
                old_proc: GetWindowLongPtrW(handle, GWL_WNDPROC),
            })
        };
        unsafe {
            assert_ne!(
                SetWindowLongPtrW(data.handle, GWL_WNDPROC, window_proc as isize),
                0
            );
            SetWindowLongPtrW(data.handle, GWLP_USERDATA, data.as_mut() as *mut _ as isize);

            let mut title = "Solar Screen Brightness".to_wide();
            SetWindowTextW(data.handle, PWSTR(title.as_mut_ptr()));
            let mut asset = Assets::get("icon-256.png")
                .expect("Icon missing")
                .into_owned();
            let hicon = CreateIconFromResource(
                asset.as_mut_ptr(),
                asset.len() as u32,
                BOOL::from(true),
                0x00030000,
            );
            SendMessageW(
                data.handle,
                WM_SETICON,
                WPARAM(ICON_BIG as usize),
                LPARAM(hicon.0),
            );
            SendMessageW(
                data.handle,
                WM_SETICON,
                WPARAM(ICON_SMALL as usize),
                LPARAM(hicon.0),
            );
        }
        self.window_data = Some(data);
    }
}

impl WindowData {
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.handle, SW_RESTORE);
            BringWindowToTop(self.handle);
            SetForegroundWindow(self.handle);
        }
    }

    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.handle, SW_HIDE);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let window_data: &mut WindowData = get_user_data(&hwnd).unwrap();
    match msg {
        WM_CLOSE => {
            window_data.hide();
            return LRESULT(0);
        }
        WM_SYSCOMMAND => {
            if wparam == WPARAM(SC_CLOSE as usize) {
                window_data.hide();
                return LRESULT(0);
            }
        }
        _ => {}
    };
    let ptr = window_data.old_proc as *const ();
    let code: WNDPROC = std::mem::transmute(ptr);
    CallWindowProcW(Some(code), hwnd, msg, wparam, lparam)
}

fn await_handle() -> HWND {
    extern "C" {
        // https://github.com/Bill-Gray/PDCursesMod/blob/44e38bbbdad146144d91fb6c28536975c27be54e/wingui/pdcscrn.c#L105
        // https://github.com/Bill-Gray/PDCursesMod/blob/master/wingui/pdcscrn.c#L2097
        static PDC_hWnd: HWND;
    }
    loop {
        unsafe {
            if !PDC_hWnd.is_null() {
                return PDC_hWnd;
            }
        }
    }
}
