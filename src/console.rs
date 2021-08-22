use crate::assets::Assets;
use crate::controller::StateRef;
use crate::tray::TrayMessageSender;
use crate::tui::run;
use crate::wide::{wchar_to_string, WideString};

use solar_screen_brightness_windows_bindings::Windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, PWSTR, WPARAM},
    System::Threading::GetCurrentProcessId,
    UI::WindowsAndMessaging::{
        BringWindowToTop, CreateIconFromResource, EnumWindows, GetClassNameW,
        GetWindowThreadProcessId, SendMessageW, SetForegroundWindow, SetWindowTextW, ShowWindow,
        ICON_BIG, ICON_SMALL, SW_HIDE, SW_RESTORE, WM_SETICON,
    },
};

pub struct Console {
    handle: HWND,
}

impl Console {
    pub fn create(tray: TrayMessageSender, state: StateRef) -> Self {
        std::thread::spawn(move || {
            run(tray, state);
        });
        let console = Console {
            handle: find_console_handle(),
        };
        console.configure();
        console.show();
        console
    }

    fn configure(&self) {
        unsafe {
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
            assert!(!hicon.is_null());
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

// unsafe extern "system" fn tray_window_proc(
//     hwnd: HWND,
//     msg: u32,
//     w_param: WPARAM,
//     l_param: LPARAM,
// ) -> LRESULT {
//     println!("{}", msg);
//     return DefWindowProcW(hwnd, msg, w_param, l_param);
// }

struct FindParam {
    pid: u32,
    handle: HWND,
}

fn find_console_handle() -> HWND {
    unsafe {
        let mut find = FindParam {
            pid: GetCurrentProcessId(),
            handle: HWND::NULL,
        };
        while find.handle.is_null() {
            println!("trying");
            EnumWindows(Some(enum_proc), LPARAM((&mut find as *mut _) as isize));
        }
        println!("got");
        find.handle
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
        if class_name == "Curses_App" {
            find.handle = window;
            return false.into();
        }
    }
    true.into()
}
