use crate::assets::Assets;
use crate::wide::WideString;
use crate::console::Console;
use winapi::um::winnt::LPCWSTR;
use winapi::um::errhandlingapi::{SetLastError, GetLastError};
use winapi::um::winuser::{DefWindowProcW, CreateWindowExW, WNDCLASSW, RegisterClassW, CW_USEDEFAULT, WS_OVERLAPPEDWINDOW, CreateIconFromResource, WM_APP, WM_LBUTTONUP, WM_RBUTTONUP, GetMessageW, TranslateMessage, DispatchMessageW, SetWindowLongPtrW, GWLP_USERDATA, GetWindowLongPtrW, SendMessageW, PostQuitMessage};
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT, TRUE, HINSTANCE, LOWORD, DWORD};
use winapi::shared::windef::{HWND, HMENU, HICON};
use winapi::um::shellapi::{NIM_ADD, NOTIFYICONDATAW, Shell_NotifyIconW, NIF_MESSAGE, NIF_ICON, NIF_TIP, NIM_DELETE};
use winapi::shared::ntdef::{NULL};
use winapi::um::libloaderapi::GetModuleHandleW;

const CALLBACK_MSG: UINT = WM_APP + 1;
const CLOSE_CONSOLE_MSG: UINT = WM_APP + 2;
const EXIT_APPLICATION_MSG: UINT = WM_APP + 3;

#[derive(Debug)]
pub enum TrayMessage {
    CloseConsole,
    ExitApplication,
}

pub type MessageSender = Box<dyn Fn(TrayMessage) + Send + Sync>;

impl TrayMessage {

    fn send(&self, hwnd: HWND) {
        let msg = match &self {
            TrayMessage::CloseConsole => { CLOSE_CONSOLE_MSG },
            TrayMessage::ExitApplication => { EXIT_APPLICATION_MSG },
        };
        unsafe {
            SendMessageW(hwnd, msg, 0, 0);
        }
    }

}

struct WindowData {
    icon: NOTIFYICONDATAW,
    console: Option<Console>,
}

pub struct TrayApplication {
    window_data: Box<WindowData>
}

impl TrayApplication {

    pub fn create() -> Self {
        unsafe {
            let hinstance = GetModuleHandleW( NULL as LPCWSTR );
            if hinstance == NULL as HINSTANCE { panic!("Get hinstance failed") }

            let mut window_class: WNDCLASSW =  std::mem::MaybeUninit::zeroed().assume_init();
            window_class.lpfnWndProc = Some(tray_window_proc);
            window_class.hInstance = hinstance;
            window_class.lpszClassName = "TrayHolder".to_wide().as_ptr();
            let atom = RegisterClassW(&window_class);
            if atom == 0 { panic!("Register window class failed") }

            let hwnd = CreateWindowExW(
                0,
                atom as *const u16,
                "tray".to_wide().as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                NULL as HWND,
                NULL as HMENU,
                hinstance,
                NULL);
            if hwnd == NULL as HWND { panic!("Create window failed") }

            let mut asset = Assets::get("icon-256.png").expect("Icon missing").into_owned();
            let hicon = CreateIconFromResource(asset.as_mut_ptr(), asset.len() as u32, TRUE, 0x00030000);
            if hicon == NULL as HICON { panic!("Failed to create icon") }

            let mut data: NOTIFYICONDATAW =  std::mem::MaybeUninit::zeroed().assume_init();
            let mut name = "Solar Screen Brightness".to_wide();
            name.resize(data.szTip.len(), 0);
            let bytes = &name[..data.szTip.len()];
            data.hWnd = hwnd;
            data.hIcon = hicon;
            data.uCallbackMessage = CALLBACK_MSG;
            data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            data.szTip.copy_from_slice(bytes);
            if Shell_NotifyIconW(NIM_ADD, &mut data) != TRUE { panic!("Error creating tray icon") }

            let mut app = TrayApplication {
                window_data: Box::new(WindowData {
                    icon: data,
                    console: None
                })
            };
            SetLastError(0);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, app.window_data.as_mut() as *mut _ as isize);
            if GetLastError() != 0 { panic!("Failed to set GWLP_USERDATA") }

            app
        }
    }

    pub fn run(&self) {
        unsafe {
            let mut msg = std::mem::MaybeUninit::uninit().assume_init();
            loop {
                let ret = GetMessageW(&mut msg, self.window_data.icon.hWnd, 0, 0);
                match ret {
                    -1 => { panic!("GetMessage failed"); }
                    0 => { break }
                    _ => {
                        TranslateMessage(&mut msg);
                        DispatchMessageW(&mut msg);
                    }
                }
            }
        }
    }
}

impl Drop for WindowData {

    fn drop(&mut self) {
        unsafe {
            if Shell_NotifyIconW(NIM_DELETE, &mut self.icon) != TRUE { panic!("Error removing tray icon") };
        }
    }
}

unsafe fn get_user_data(hwnd: &HWND) -> &mut WindowData {
    let user_data = GetWindowLongPtrW(*hwnd, GWLP_USERDATA);
    if user_data == 0 { panic!("Get GWLP_USERDATA failed") }
    &mut *(user_data as *mut WindowData)
}

unsafe extern "system" fn tray_window_proc(hwnd: HWND, msg: UINT, w_param : WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        CALLBACK_MSG => {
            match LOWORD(l_param as DWORD) as u32 {
                WM_LBUTTONUP | WM_RBUTTONUP  => {
                    let app = get_user_data(&hwnd);
                    let hwnd = app.icon.hWnd as usize;
                    match &app.console {
                        Some(c) => { c.show(); }
                        None => {
                            app.console = Some(Console::create(
                                Box::new(move |msg| { msg.send(hwnd as HWND) })
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        CLOSE_CONSOLE_MSG => {
            let app = get_user_data(&hwnd);
            app.console.as_ref().unwrap().hide();
        }
        EXIT_APPLICATION_MSG => {
            PostQuitMessage(0);
        }
        _ => {}
    }
    return DefWindowProcW( hwnd , msg , w_param , l_param );
}
