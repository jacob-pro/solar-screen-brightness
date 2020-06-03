use crate::assets::Assets;
use crate::wide::WideString;
use crate::console::Console;
use winapi::um::winnt::LPCWSTR;
use winapi::um::errhandlingapi::{SetLastError, GetLastError};
use winapi::um::winuser::*;
use winapi::um::shellapi::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::shared::ntdef::{NULL};
use crate::brightness::{BrightnessMessageSender, BrightnessStatusRef, BrightnessLoopMessage};

extern "system" {
    pub fn WTSRegisterSessionNotification(hwnd: HWND, flags: DWORD) -> BOOL;
}

const CALLBACK_MSG: UINT = WM_APP + 1;
const CLOSE_CONSOLE_MSG: UINT = WM_APP + 2;
const EXIT_APPLICATION_MSG: UINT = WM_APP + 3;

#[derive(Debug)]
pub enum TrayMessage {
    CloseConsole,
    ExitApplication,
}

pub type TrayMessageSender = Box<dyn Fn(TrayMessage) + Send + Sync>;

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
    sender: BrightnessMessageSender,
    status: BrightnessStatusRef,
    prev_running: bool,
}

// Blocking call, runs on this thread
pub fn run(sender: BrightnessMessageSender, status: BrightnessStatusRef) {
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

        if WTSRegisterSessionNotification(hwnd, 0) != TRUE { panic!("Failed to WTSRegisterSessionNotification")}

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

        let mut window_data = Box::new(WindowData {
            icon: data,
            console: None,
            sender,
            status,
            prev_running: false,
        });
        SetLastError(0);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_data.as_mut() as *mut _ as isize);
        if GetLastError() != 0 { panic!("Failed to set GWLP_USERDATA") }

        let mut msg = std::mem::MaybeUninit::uninit().assume_init();
        loop {
            let ret = GetMessageW(&mut msg, NULL as HWND, 0, 0);
            match ret {
                -1 => { panic!("GetMessage failed"); }
                0 => { break }
                _ => {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                }
            }
        }

        if Shell_NotifyIconW(NIM_DELETE, &mut window_data.icon) != TRUE { panic!("Error removing tray icon") };
    }
}

unsafe fn get_user_data(hwnd: &HWND) -> Option<&mut WindowData> {
    let user_data = GetWindowLongPtrW(*hwnd, GWLP_USERDATA);
    if user_data == 0 { return None }
    Some(&mut *(user_data as *mut WindowData))
}

unsafe extern "system" fn tray_window_proc(hwnd: HWND, msg: UINT, w_param : WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        CALLBACK_MSG => {
            match LOWORD(l_param as DWORD) as u32 {
                WM_LBUTTONUP | WM_RBUTTONUP  => {
                    let app = get_user_data(&hwnd).unwrap();
                    let hwnd = app.icon.hWnd as usize;
                    match &app.console {
                        Some(c) => { c.show(); }
                        None => {
                            app.console = Some(Console::create(
                                Box::new(move |msg| { msg.send(hwnd as HWND) }),
                                app.sender.clone(),
                                app.status.clone()
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        CLOSE_CONSOLE_MSG => {
            let app = get_user_data(&hwnd).unwrap();
            app.console.as_ref().unwrap().hide();
        }
        EXIT_APPLICATION_MSG => {
            PostQuitMessage(0);
        },
        WM_WTSSESSION_CHANGE => {
            let app = get_user_data(&hwnd).unwrap();
            match w_param {
                WTS_SESSION_LOCK => {
                    app.prev_running = *app.status.read().unwrap().running();
                    app.sender.send(BrightnessLoopMessage::Pause).unwrap();
                },
                WTS_SESSION_UNLOCK => {
                    if app.prev_running {
                        app.sender.send(BrightnessLoopMessage::Resume).unwrap();
                    }
                },
                _ => {}
            }

        }
        _ => {}
    }
    return DefWindowProcW( hwnd , msg , w_param , l_param );
}
