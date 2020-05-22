#![windows_subsystem = "windows"]
#![allow(unused_imports)]

use cursive::views::{Dialog, TextView};
use cursive::Cursive;
use winapi::um::consoleapi::{AllocConsole, SetConsoleCtrlHandler};
use winapi::um::winuser::{GetTopWindow, TranslateMessage, GetMessageW, LPMSG, GetMessageA, DispatchMessageA, DefWindowProcW, CreateWindowExW, LPWNDCLASSW, WNDCLASSW, RegisterClassW, CW_USEDEFAULT, WS_OVERLAPPEDWINDOW};
use winapi::_core::ptr::{null_mut};
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT, DWORD, BOOL, TRUE, FALSE, HINSTANCE, WORD, ATOM};
use winapi::shared::windef::{HWND, HMENU};
use winapi::shared::guiddef::GUID;
use winapi::um::wincon::{FreeConsole, CTRL_CLOSE_EVENT};
use winapi::um::processthreadsapi::ExitThread;
use winapi::um::shellapi::{NIM_ADD, NOTIFYICONDATAW, Shell_NotifyIconW, NIF_MESSAGE};
use winapi::shared::ntdef::{NULL, LPWSTR};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winnt::LPCWSTR;

fn wide_encode(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::iter::once;
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}


fn main() {

    unsafe {
        let hinstance = GetModuleHandleW( null_mut() );
        if hinstance == NULL as HINSTANCE { panic!("Get hinstance failed")};

        let mut window_class: WNDCLASSW =  std::mem::MaybeUninit::zeroed().assume_init();
        window_class.lpfnWndProc = Some(window_procedure);
        window_class.hInstance = hinstance;
        window_class.lpszClassName = wide_encode("TrayHolder").as_ptr();
        let atom = RegisterClassW(&window_class);
        if atom == 0 { panic!("Register window class failed")};

        let hwnd = CreateWindowExW(
            0,
            atom as *const u16,
            wide_encode("tray").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            NULL as HWND,
            NULL as HMENU,
            hinstance,
            NULL);
        if hwnd == NULL as HWND { panic!("Create window failed")};

        let mut data: NOTIFYICONDATAW =  std::mem::MaybeUninit::zeroed().assume_init();
        data.hWnd = hwnd;
        if Shell_NotifyIconW(NIM_ADD, &mut data) != TRUE { panic!("Error creating tray icon") };
    }

    // // Creates the cursive root - required for every application.
    // let mut siv = Cursive::crossterm().unwrap();
    //
    // // Creates a dialog with a single "Quit" button
    // siv.add_layer(Dialog::around(TextView::new("Hello Dialog!"))
    //     .title("Cursive")
    //     .button("Quit", |s| s.quit()));
    //
    //
    // // Starts the event loop.
    // siv.run();



    unsafe {
        if AllocConsole() != TRUE {panic!("Error opening console")};
        if SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) != TRUE {panic!("Error setting handler")};
    }

    loop {
        let x = 5;
    }
}

// When the console window is being closed
unsafe extern "system" fn ctrl_handler(dw_ctrl_type: DWORD) -> BOOL {
    FreeConsole();
    // Prevent ExitProcess by not returning the handler
    if dw_ctrl_type == CTRL_CLOSE_EVENT {
        ExitThread(0);
    }
    TRUE
}


unsafe extern "system" fn window_procedure(hwnd: HWND, msg: UINT, w_param : WPARAM, l_param: LPARAM) -> LRESULT {
    return DefWindowProcW( hwnd , msg , w_param , l_param );
}
