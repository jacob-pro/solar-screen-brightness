use crate::assets::Assets;
use crate::controller::StateRef;
use crate::tray::TrayMessageSender;
use crate::tui::run;
use crate::wide::{wchar_to_string, WideString};

use solar_screen_brightness_windows_bindings::Windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, PWSTR, WPARAM},
    System::Threading::GetCurrentProcessId,
    UI::WindowsAndMessaging::{
        BringWindowToTop, CallWindowProcW, CreateIconFromResource, EnumWindows, GetClassNameW,
        GetWindowLongPtrW, GetWindowThreadProcessId, SendMessageW, SetForegroundWindow,
        SetWindowLongPtrW, SetWindowTextW, ShowWindow, GWLP_USERDATA, GWL_WNDPROC, ICON_BIG,
        ICON_SMALL, SW_HIDE, SW_RESTORE, WM_CLOSE, WM_SETICON, WM_SYSCOMMAND, WNDPROC,
    },
};

pub struct Console {
    handle: HWND,
    old_proc: isize,
    old_data: isize,
}

impl Console {
    pub fn create(tray: TrayMessageSender, state: StateRef) -> Self {
        std::thread::spawn(move || {
            run(tray, state);
        });
        let handle = find_console_handle();
        let mut console = unsafe {
            Console {
                handle,
                old_proc: GetWindowLongPtrW(handle, GWL_WNDPROC),
                old_data: GetWindowLongPtrW(handle, GWLP_USERDATA),
            }
        };
        console.configure();
        console.show();
        console
    }

    fn configure(&mut self) {
        unsafe {
            assert_ne!(
                SetWindowLongPtrW(self.handle, GWL_WNDPROC, window_proc as isize),
                0
            );
            SetWindowLongPtrW(self.handle, GWLP_USERDATA, (self as *mut _) as isize);

            let mut title = "Solar Screen Brightness".to_wide();
            SetWindowTextW(self.handle, PWSTR(title.as_mut_ptr()));
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
                self.handle,
                WM_SETICON,
                WPARAM(ICON_BIG as usize),
                LPARAM(hicon.0),
            );
            SendMessageW(
                self.handle,
                WM_SETICON,
                WPARAM(ICON_SMALL as usize),
                LPARAM(hicon.0),
            );
        }
    }

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

unsafe fn get_user_data<T>(hwnd: &HWND) -> Option<&mut T> {
    let user_data = GetWindowLongPtrW(*hwnd, GWLP_USERDATA);
    if user_data == 0 {
        return None;
    }
    Some(&mut *(user_data as *mut T))
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => return LRESULT(0),
        WM_SYSCOMMAND => return LRESULT(0),
        _ => {}
    };
    let console: &mut Console = get_user_data(&hwnd).unwrap();
    let ptr = console.old_proc as *const ();
    let code: WNDPROC = std::mem::transmute(ptr);
    CallWindowProcW(Some(code), hwnd, msg, wparam, lparam)
}

struct FindParam {
    pid: u32,
    handle: HWND,
}

const CURSES_CLASS: &'static str = "Curses_App";

fn find_console_handle() -> HWND {
    unsafe {
        let mut find = FindParam {
            pid: GetCurrentProcessId(),
            handle: HWND::NULL,
        };
        for _ in 0..100 {
            EnumWindows(Some(enum_proc), LPARAM((&mut find as *mut _) as isize));
            if !find.handle.is_null() {
                return find.handle;
            }
        }
        panic!("Unable to find Window of class '{}'", CURSES_CLASS);
    }
}

unsafe extern "system" fn enum_proc(window: HWND, arg: LPARAM) -> BOOL {
    let mut find = &mut *(arg.0 as *mut FindParam);
    let mut pid = 0;
    GetWindowThreadProcessId(window, &mut pid);
    if pid == find.pid {
        let mut buffer: Vec<u16> = Vec::with_capacity(120);
        let len = GetClassNameW(window, PWSTR(buffer.as_mut_ptr()), buffer.capacity() as i32);
        buffer.set_len(len as usize);
        let class_name = wchar_to_string(buffer.as_slice());
        if class_name == CURSES_CLASS {
            find.handle = window;
            return false.into();
        }
    }
    true.into()
}
