use crate::assets::Assets;
use crate::console::Console;
use crate::controller::{BrightnessController, StateRef};
use crate::wide::{get_user_data, WideString};

use crate::wide;
use solar_screen_brightness_windows_bindings::Windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, PWSTR, WPARAM},
    System::Diagnostics::Debug::{GetLastError, SetLastError, WIN32_ERROR},
    System::LibraryLoader::GetModuleHandleW,
    System::RemoteDesktop::WTSRegisterSessionNotification,
    UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    },
    UI::WindowsAndMessaging::{
        CreateIconFromResource, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
        PostQuitMessage, RegisterClassW, SendMessageW, SetWindowLongPtrW, TranslateMessage,
        CW_USEDEFAULT, GWLP_USERDATA, HMENU, WINDOW_EX_STYLE, WM_APP, WM_LBUTTONUP, WM_RBUTTONUP,
        WM_WTSSESSION_CHANGE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
    },
};

const CALLBACK_MSG: u32 = WM_APP + 1;
const CLOSE_CONSOLE_MSG: u32 = WM_APP + 2;
const EXIT_APPLICATION_MSG: u32 = WM_APP + 3;

#[derive(Debug)]
pub enum TrayMessage {
    CloseConsole,
    ExitApplication,
}

pub type TrayMessageSender = Box<dyn Fn(TrayMessage) + Send + Sync>;

impl TrayMessage {
    fn send(&self, hwnd: HWND) {
        let msg = match &self {
            TrayMessage::CloseConsole => CLOSE_CONSOLE_MSG,
            TrayMessage::ExitApplication => EXIT_APPLICATION_MSG,
        };
        unsafe {
            SendMessageW(hwnd, msg, WPARAM(0), LPARAM(0));
        }
    }
}

struct WindowData {
    icon: NOTIFYICONDATAW,
    console: Option<Console>,
    state: StateRef,
    prev_running: bool,
}

// Blocking call, runs on this thread
pub fn run(controller: &BrightnessController) {
    unsafe {
        let hinstance = GetModuleHandleW(PWSTR::NULL);
        assert!(!hinstance.is_null());

        let mut window_class: WNDCLASSW = std::mem::MaybeUninit::zeroed().assume_init();
        window_class.lpfnWndProc = Some(tray_window_proc);
        window_class.hInstance = hinstance;
        let mut name = "TrayHolder".to_wide();
        window_class.lpszClassName = PWSTR(name.as_mut_ptr());
        let atom = RegisterClassW(&window_class);
        assert_ne!(atom, 0);

        let mut name = "tray".to_wide();
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PWSTR(atom as *mut u16),
            PWSTR(name.as_mut_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            HWND::NULL,
            HMENU::NULL,
            hinstance,
            std::ptr::null_mut(),
        );
        assert!(!hwnd.is_null());

        assert!(WTSRegisterSessionNotification(hwnd, 0).as_bool());

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

        let mut data: NOTIFYICONDATAW = std::mem::MaybeUninit::zeroed().assume_init();
        let mut name = "Solar Screen Brightness".to_wide();
        name.resize(data.szTip.len(), 0);
        let bytes = &name[..data.szTip.len()];
        data.hWnd = hwnd;
        data.hIcon = hicon;
        data.uCallbackMessage = CALLBACK_MSG;
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.szTip.copy_from_slice(bytes);
        assert!(Shell_NotifyIconW(NIM_ADD, &mut data).as_bool());

        let mut window_data = Box::new(WindowData {
            icon: data,
            console: None,
            state: controller.state.clone(),
            prev_running: false,
        });
        SetLastError(0);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_data.as_mut() as *mut _ as isize);
        assert_eq!(
            GetLastError(),
            WIN32_ERROR(0),
            "Failed to set GWLP_USERDATA"
        );

        let mut msg = std::mem::MaybeUninit::uninit().assume_init();
        loop {
            let ret = GetMessageW(&mut msg, HWND::NULL, 0, 0).0;
            match ret {
                -1 => {
                    panic!("GetMessage failed");
                }
                0 => break,
                _ => {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                }
            }
        }

        assert!(Shell_NotifyIconW(NIM_DELETE, &mut window_data.icon).as_bool());
    }
}

unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        CALLBACK_MSG => match wide::loword(l_param.0 as u32) {
            WM_LBUTTONUP | WM_RBUTTONUP => {
                let app = get_user_data::<WindowData>(&hwnd).unwrap();
                let hwnd = app.icon.hWnd;
                match &app.console {
                    Some(c) => {
                        c.show();
                    }
                    None => {
                        app.console = Some(Console::create(
                            Box::new(move |msg| msg.send(hwnd)),
                            app.state.clone(),
                        ));
                    }
                }
            }
            _ => {}
        },
        CLOSE_CONSOLE_MSG => {
            let app = get_user_data::<WindowData>(&hwnd).unwrap();
            app.console.as_ref().unwrap().hide();
        }
        EXIT_APPLICATION_MSG => {
            PostQuitMessage(0);
        }
        WM_WTSSESSION_CHANGE => {
            let app = get_user_data::<WindowData>(&hwnd).unwrap();
            match w_param.0 as u32 {
                WTS_SESSION_LOCK => {
                    let mut state = app.state.write().unwrap();
                    app.prev_running = state.get_enabled();
                    state.set_enabled(false);
                }
                WTS_SESSION_UNLOCK => {
                    if app.prev_running {
                        app.state.write().unwrap().set_enabled(true);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    return DefWindowProcW(hwnd, msg, w_param, l_param);
}
