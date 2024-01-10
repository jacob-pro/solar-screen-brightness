use crate::controller::{BrightnessController, Message};
use crate::gui::UserEvent;
use egui_winit::winit::event_loop::{EventLoop, EventLoopProxy};
use std::sync::mpsc;
use std::sync::mpsc::sync_channel;
use std::thread::JoinHandle;
use win32_utils::error::{check_error, CheckError};
use win32_utils::window::WindowDataExtension;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::RemoteDesktop::WTSRegisterSessionNotification;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcA, DispatchMessageA, GetMessageA, PostQuitMessage,
    RegisterClassW, RegisterWindowMessageW, SendMessageW, SetWindowLongPtrW, CW_USEDEFAULT,
    GWLP_USERDATA, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_DISPLAYCHANGE,
    WM_WTSSESSION_CHANGE, WNDCLASSW, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};

const EXIT_LOOP: u32 = WM_APP + 999;

pub struct EventWatcher {
    thread: Option<JoinHandle<()>>,
    hwnd: HWND,
}

impl EventWatcher {
    pub fn start(
        controller: &BrightnessController,
        main_loop: Option<&EventLoop<UserEvent>>,
    ) -> anyhow::Result<Self> {
        let brightness_sender = controller.sender.clone();
        let proxy = main_loop.map(|m| m.create_proxy());
        let (tx, rx) = sync_channel(0);

        let thread = std::thread::spawn(move || {
            let mut window_data = Box::new(WindowData {
                sender: brightness_sender,
                open_window_msg_code: register_open_window_message(),
                main_loop: proxy,
            });

            unsafe {
                // Create Window Class
                let instance = GetModuleHandleW(None).unwrap();
                let window_class = WNDCLASSW {
                    hInstance: instance.into(),
                    lpszClassName: w!("ssb_event_watcher"),
                    lpfnWndProc: Some(wndproc),
                    ..Default::default()
                };
                let atom = check_error(|| RegisterClassW(&window_class)).unwrap();

                // Create window
                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    PCWSTR(atom as *const u16),
                    None,
                    WINDOW_STYLE::default(),
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    None,
                    None,
                    instance,
                    None,
                )
                .check_error()
                .unwrap();

                // Register Window data
                check_error(|| {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_data.as_mut() as *mut _ as isize)
                })
                .unwrap();

                tx.send(hwnd).unwrap();

                // Register for Session Notifications
                WTSRegisterSessionNotification(hwnd, 0).unwrap();

                let mut message = MSG::default();
                while GetMessageA(&mut message, hwnd, 0, 0).into() {
                    DispatchMessageA(&message);
                }
            }
            log::debug!("EventWatcher thread exiting");
        });

        let hwnd = rx.recv().unwrap();
        Ok(EventWatcher {
            thread: Some(thread),
            hwnd,
        })
    }
}

impl Drop for EventWatcher {
    fn drop(&mut self) {
        log::info!("Stopping EventWatcher");
        unsafe { SendMessageW(self.hwnd, EXIT_LOOP, None, None) };
        self.thread.take().unwrap().join().unwrap();
    }
}

struct WindowData {
    sender: mpsc::Sender<Message>,
    open_window_msg_code: u32,
    main_loop: Option<EventLoopProxy<UserEvent>>,
}

unsafe extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if let Some(window_data) = window.get_user_data::<WindowData>() {
        match message {
            WM_DISPLAYCHANGE => {
                log::info!("Detected possible display change (WM_DISPLAYCHANGE)");
                window_data
                    .sender
                    .send(Message::Refresh("WM_DISPLAYCHANGE"))
                    .unwrap();
            }
            EXIT_LOOP => {
                PostQuitMessage(0);
            }
            WM_WTSSESSION_CHANGE => match wparam.0 as u32 {
                WTS_SESSION_LOCK => {
                    log::info!("Detected WTS_SESSION_LOCK");
                    window_data
                        .sender
                        .send(Message::Disable("WTS_SESSION_LOCK"))
                        .unwrap();
                }
                WTS_SESSION_UNLOCK => {
                    log::info!("Detected WTS_SESSION_UNLOCK");
                    window_data
                        .sender
                        .send(Message::Enable("WTS_SESSION_UNLOCK"))
                        .unwrap();
                }
                _ => {}
            },
            msg if msg == window_data.open_window_msg_code => {
                if let Some(event_loop) = &window_data.main_loop {
                    log::info!("Opening window due to external message");
                    event_loop
                        .send_event(UserEvent::OpenWindow("Broadcast Message"))
                        .unwrap();
                }
            }
            _ => {}
        }
    }
    DefWindowProcA(window, message, wparam, lparam)
}

pub fn register_open_window_message() -> u32 {
    unsafe {
        check_error(|| RegisterWindowMessageW(w!("solar-screen-brightness.open_window"))).unwrap()
    }
}
