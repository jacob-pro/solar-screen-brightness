use crate::assets::Assets;
use crate::controller::StateRef;
use crate::tray::TrayMessageSender;
use crate::tui::run;
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

pub struct Console(Box<WindowData>);

impl Console {
    pub fn create(tray: TrayMessageSender, state: StateRef) -> Self {
        std::thread::spawn(move || {
            run(tray, state);
        });
        let handle = await_handle();
        let mut console = unsafe {
            Console(Box::new(WindowData {
                handle,
                old_proc: GetWindowLongPtrW(handle, GWL_WNDPROC),
            }))
        };
        console.configure();
        console.show();
        console
    }

    fn configure(&mut self) {
        unsafe {
            assert_ne!(
                SetWindowLongPtrW(self.0.handle, GWL_WNDPROC, window_proc as isize),
                0
            );
            SetWindowLongPtrW(
                self.0.handle,
                GWLP_USERDATA,
                self.0.as_mut() as *mut _ as isize,
            );

            let mut title = "Solar Screen Brightness".to_wide();
            SetWindowTextW(self.0.handle, PWSTR(title.as_mut_ptr()));
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
                self.0.handle,
                WM_SETICON,
                WPARAM(ICON_BIG as usize),
                LPARAM(hicon.0),
            );
            SendMessageW(
                self.0.handle,
                WM_SETICON,
                WPARAM(ICON_SMALL as usize),
                LPARAM(hicon.0),
            );
        }
    }

    pub fn show(&self) {
        self.0.show();
    }

    pub fn hide(&self) {
        self.0.hide();
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
