#![windows_subsystem = "windows"]
#![allow(unused_imports)]

use cursive::views::{Dialog, TextView};
use cursive::Cursive;
use winapi::um::consoleapi::{AllocConsole, SetConsoleCtrlHandler};
use winapi::um::winuser::{GetTopWindow, TranslateMessage, GetMessageW, LPMSG, GetMessageA, DispatchMessageA, DefWindowProcW};
use winapi::_core::ptr::{null_mut};
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT, DWORD, BOOL, TRUE, FALSE};
use winapi::shared::windef::{HWND};
use winapi::um::wincon::{FreeConsole, CTRL_CLOSE_EVENT};
use winapi::um::processthreadsapi::ExitThread;


fn main() {

    unsafe {
        if AllocConsole() != TRUE {panic!("Error opening console")};
        if SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) != TRUE {panic!("Error setting handler")};
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
