use crate::assets::Assets;
use crate::console::Console;
use crate::controller::BrightnessController;
use crate::cursive::Cursive;
use crate::lock::ApplicationLock;
use crate::tray::TrayApplicationHandle;
use crate::APP_NAME;
use anyhow::Context;
use solar_screen_brightness_windows::{loword, set_and_get_error, WideString, WindowDataExtension};
use std::panic::PanicInfo;
use std::sync::Arc;
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, PWSTR, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    System::RemoteDesktop::WTSRegisterSessionNotification,
    UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    },
    UI::WindowsAndMessaging::{
        CreateIconFromResource, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
        MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW,
        SendMessageW, SetWindowLongPtrW, TranslateMessage, CW_USEDEFAULT, GWLP_USERDATA, HMENU,
        MB_ICONSTOP, MB_OK, MSG, WM_APP, WM_DISPLAYCHANGE, WM_LBUTTONUP, WM_RBUTTONUP,
        WM_WTSSESSION_CHANGE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
    },
};

const SHOW_CONSOLE_MSG: &str = "solar-screen-brightness.show_console";
const CALLBACK_MSG: u32 = WM_APP + 1;
const CLOSE_CONSOLE_MSG: u32 = WM_APP + 2;
const EXIT_APPLICATION_MSG: u32 = WM_APP + 3;

struct WindowData {
    console: Console,
    controller: Arc<BrightnessController>,
    prev_running: bool,
    show_console_msg_code: u32,
}

pub fn run(controller: Arc<BrightnessController>, _lock: ApplicationLock, launch_console: bool) {
    std::panic::set_hook(Box::new(handle_panic));
    unsafe {
        // Create Window Class
        let hinstance = set_and_get_error(|| GetModuleHandleW(PWSTR::default())).unwrap();
        let mut name = "TrayHolder".to_wide();
        let window_class = WNDCLASSW {
            lpfnWndProc: Some(tray_window_proc),
            hInstance: hinstance,
            lpszClassName: PWSTR(name.as_mut_ptr()),
            ..Default::default()
        };
        let atom = set_and_get_error(|| RegisterClassW(&window_class)).unwrap();

        // Create Window
        let hwnd = set_and_get_error(|| {
            CreateWindowExW(
                0,
                PWSTR(atom as *mut u16),
                "tray",
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND::default(),
                HMENU::default(),
                hinstance,
                std::ptr::null_mut(),
            )
        })
        .unwrap();

        // Register Window data
        let mut window_data = Box::new(WindowData {
            console: Console::new(TrayApplicationHandle(Handle(hwnd)), Arc::clone(&controller)),
            controller,
            prev_running: false,
            show_console_msg_code: RegisterWindowMessageW(SHOW_CONSOLE_MSG),
        });
        set_and_get_error(|| {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_data.as_mut() as *mut _ as isize)
        })
        .unwrap();

        // Register for Session Notifications
        WTSRegisterSessionNotification(hwnd, 0).ok().unwrap();

        // Create hicon
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

        // Create tray icon
        let mut tray_icon: NOTIFYICONDATAW = std::mem::MaybeUninit::zeroed().assume_init();
        let mut name = APP_NAME.to_wide();
        name.resize(tray_icon.szTip.len(), 0);
        let bytes = &name[..tray_icon.szTip.len()];
        tray_icon.hWnd = hwnd;
        tray_icon.hIcon = hicon;
        tray_icon.uCallbackMessage = CALLBACK_MSG;
        tray_icon.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        tray_icon.szTip.copy_from_slice(bytes);
        Shell_NotifyIconW(NIM_ADD, &tray_icon).ok().unwrap();

        if launch_console {
            window_data.console.show();
        }

        // Start run loop
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, HWND::default(), 0, 0).0;
            match ret {
                -1 => {
                    panic!("GetMessage failed");
                }
                0 => break,
                _ => {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }

        // Cleanup
        Shell_NotifyIconW(NIM_DELETE, &tray_icon).ok().unwrap();
    }
}

unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match hwnd.get_user_data::<WindowData>() {
        None => {}
        Some(app) => match msg {
            CALLBACK_MSG => match loword(l_param as u32) {
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
            WM_WTSSESSION_CHANGE => match w_param as u32 {
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
            WM_DISPLAYCHANGE => {
                log::info!("Displays changed, doing refresh");
                app.controller.force_refresh();
            }
            msg if msg == app.show_console_msg_code => {
                app.console.show();
            }
            _ => {}
        },
    }
    DefWindowProcW(hwnd, msg, w_param, l_param)
}

#[derive(Clone)]
pub(super) struct Handle(HWND);

impl Handle {
    fn send_message(&self, msg: u32) {
        unsafe {
            SendMessageW(self.0, msg, 0, 0);
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
        let text = format!("{}", info);
        MessageBoxW(
            HWND::default(),
            text.as_str(),
            "Fatal Error",
            MB_OK | MB_ICONSTOP,
        );
        std::process::exit(1);
    }
}

pub fn show_console_in_owning_process() -> Result<(), anyhow::Error> {
    const HWND_BROADCAST: HWND = 0xffff;
    unsafe {
        let msg = set_and_get_error(|| RegisterWindowMessageW(SHOW_CONSOLE_MSG))
            .context("Registering window message")?;
        set_and_get_error(|| PostMessageW(HWND_BROADCAST, msg, 0, 0))
            .context("Posting window broadcast")?;
    }
    Ok(())
}
