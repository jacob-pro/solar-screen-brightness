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
        MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW,
        SendMessageW, SetWindowLongPtrW, TranslateMessage, CW_USEDEFAULT, GWLP_USERDATA, HMENU,
        MB_ICONSTOP, MB_OK, WINDOW_EX_STYLE, WM_APP, WM_LBUTTONUP, WM_RBUTTONUP,
        WM_WTSSESSION_CHANGE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
    },
};

const SHOW_CONSOLE_MSG: &str = "solar-screen-brightness.show_console";
const CALLBACK_MSG: u32 = WM_APP + 1;
const CLOSE_CONSOLE_MSG: u32 = WM_APP + 2;
const EXIT_APPLICATION_MSG: u32 = WM_APP + 3;

struct WindowData {
    console: Console,
    controller: BrightnessController,
    prev_running: bool,
    show_console_msg_code: u32,
}

pub fn run(controller: BrightnessController, launch_console: bool) {
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

        // Register Window data
        let mut msg_name = SHOW_CONSOLE_MSG.to_wide();
        let mut window_data = Box::new(WindowData {
            console: Console::new(TrayApplicationHandle(Handle(hwnd)), controller.clone()),
            controller,
            prev_running: false,
            show_console_msg_code: RegisterWindowMessageW(PWSTR(msg_name.as_mut_ptr())),
        });
        SetLastError(0);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_data.as_mut() as *mut _ as isize);
        assert_eq!(
            GetLastError(),
            WIN32_ERROR(0),
            "Failed to set GWLP_USERDATA"
        );

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
        let mut tray_icon: NOTIFYICONDATAW = std::mem::MaybeUninit::zeroed().assume_init();
        let mut name = "Solar Screen Brightness".to_wide();
        name.resize(tray_icon.szTip.len(), 0);
        let bytes = &name[..tray_icon.szTip.len()];
        tray_icon.hWnd = hwnd;
        tray_icon.hIcon = hicon;
        tray_icon.uCallbackMessage = CALLBACK_MSG;
        tray_icon.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        tray_icon.szTip.copy_from_slice(bytes);
        assert!(Shell_NotifyIconW(NIM_ADD, &mut tray_icon).as_bool());

        if launch_console {
            window_data.console.show();
        }

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
        assert!(Shell_NotifyIconW(NIM_DELETE, &mut tray_icon).as_bool());
    }
}

unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match get_user_data::<WindowData>(&hwnd) {
        None => {}
        Some(app) => match msg {
            CALLBACK_MSG => match loword(l_param.0 as u32) {
                WM_LBUTTONUP | WM_RBUTTONUP => {
                    app.console.show();
                }
                _ => {}
            },
            CLOSE_CONSOLE_MSG => {
                app.console.hide();
            }
            EXIT_APPLICATION_MSG => {
                PostQuitMessage(0);
            }
            WM_WTSSESSION_CHANGE => match w_param.0 as u32 {
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
            },
            msg if msg == app.show_console_msg_code => {
                app.console.show();
            }
            _ => {}
        },
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

pub fn show_console_in_another_process() {
    const HWND_BROADCAST: HWND = HWND(0xffff);
    let mut msg_name = SHOW_CONSOLE_MSG.to_wide();
    unsafe {
        let msg = RegisterWindowMessageW(PWSTR(msg_name.as_mut_ptr()));
        PostMessageW(HWND_BROADCAST, msg, WPARAM(0), LPARAM(0));
    }
}
