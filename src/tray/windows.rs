use crate::assets::Assets;
use crate::console::Console;
use crate::controller::BrightnessController;
use crate::cursive::Cursive;
use crate::tray::TrayApplicationHandle;
use crate::wide::{get_user_data, loword, WideString};
use std::panic::PanicInfo;

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
        MessageBoxW, PostQuitMessage, RegisterClassW, SendMessageW, SetWindowLongPtrW,
        TranslateMessage, CW_USEDEFAULT, GWLP_USERDATA, HMENU, MB_ICONSTOP, MB_OK, WINDOW_EX_STYLE,
        WM_APP, WM_LBUTTONUP, WM_RBUTTONUP, WM_WTSSESSION_CHANGE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
    },
};

const CALLBACK_MSG: u32 = WM_APP + 1;
const CLOSE_CONSOLE_MSG: u32 = WM_APP + 2;
const EXIT_APPLICATION_MSG: u32 = WM_APP + 3;

struct WindowData {
    tray_icon: NOTIFYICONDATAW,
    console: Console,
    controller: BrightnessController,
    prev_running: bool,
}

pub fn run(controller: BrightnessController) {
    std::panic::set_hook(Box::new(handle_panic));
    unsafe {
        // Create Window Class
        let hinstance = GetModuleHandleW(PWSTR::NULL);
        assert!(!hinstance.is_null());
        let mut window_class: WNDCLASSW = std::mem::MaybeUninit::zeroed().assume_init();
        window_class.lpfnWndProc = Some(tray_window_proc);
        window_class.hInstance = hinstance;
        let mut name = "TrayHolder".to_wide();
        window_class.lpszClassName = PWSTR(name.as_mut_ptr());
        let atom = RegisterClassW(&window_class);
        assert_ne!(atom, 0);

        // Create Window
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

        // Register for Session Notifications
        assert!(WTSRegisterSessionNotification(hwnd, 0).as_bool());

        // Create hicon
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

        // Create tray icon
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

        // Register Window data
        let mut window_data = Box::new(WindowData {
            tray_icon: data,
            console: Console::new(TrayApplicationHandle(Handle(hwnd)), controller.clone()),
            controller,
            prev_running: false,
        });
        SetLastError(0);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_data.as_mut() as *mut _ as isize);
        assert_eq!(
            GetLastError(),
            WIN32_ERROR(0),
            "Failed to set GWLP_USERDATA"
        );

        // Start run loop
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

        // Cleanup
        assert!(Shell_NotifyIconW(NIM_DELETE, &mut window_data.tray_icon).as_bool());
    }
}

unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        CALLBACK_MSG => match loword(l_param.0 as u32) {
            WM_LBUTTONUP | WM_RBUTTONUP => {
                let app = get_user_data::<WindowData>(&hwnd).unwrap();
                app.console.show();
            }
            _ => {}
        },
        CLOSE_CONSOLE_MSG => {
            let app = get_user_data::<WindowData>(&hwnd).unwrap();
            app.console.hide();
        }
        EXIT_APPLICATION_MSG => {
            PostQuitMessage(0);
        }
        WM_WTSSESSION_CHANGE => {
            let app = get_user_data::<WindowData>(&hwnd).unwrap();
            match w_param.0 as u32 {
                WTS_SESSION_LOCK => {
                    log::info!("Detected session lock, ensuring dynamic brightness disabled");
                    app.prev_running = app.controller.get_enabled();
                    app.controller.set_enabled(false);
                }
                WTS_SESSION_UNLOCK => {
                    if app.prev_running {
                        log::info!("Detected session unlock, enabling dynamic brightness");
                        app.controller.set_enabled(true);
                    } else {
                        log::info!("Detected session unlock, ignoring");
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    return DefWindowProcW(hwnd, msg, w_param, l_param);
}

#[derive(Clone)]
pub(super) struct Handle(HWND);

impl Handle {
    fn send_message(&self, msg: u32) {
        unsafe {
            SendMessageW(self.0, msg, WPARAM(0), LPARAM(0));
        }
    }

    pub(super) fn close_console(&self, _: &mut Cursive) {
        self.send_message(CLOSE_CONSOLE_MSG);
    }

    pub(super) fn exit_application(&self) {
        self.send_message(EXIT_APPLICATION_MSG);
    }
}

fn handle_panic(info: &PanicInfo) {
    log::error!("Panic: {}", info);
    unsafe {
        let mut title = "Fatal Error".to_wide();
        let mut text = format!("{}", info).as_str().to_wide();
        MessageBoxW(
            HWND::NULL,
            PWSTR(text.as_mut_ptr()),
            PWSTR(title.as_mut_ptr()),
            MB_OK | MB_ICONSTOP,
        );
        std::process::exit(1);
    }
}
