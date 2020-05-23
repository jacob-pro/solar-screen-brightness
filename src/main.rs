#![windows_subsystem = "windows"]

mod assets;
mod shell_icon;
mod str_ext;

// use cursive::views::{Dialog, TextView};
// use cursive::Cursive;
use winapi::um::consoleapi::{AllocConsole, SetConsoleCtrlHandler};
use winapi::shared::minwindef::{DWORD, BOOL, TRUE};
use winapi::um::wincon::{FreeConsole, CTRL_CLOSE_EVENT};
use winapi::um::processthreadsapi::ExitThread;
use crate::shell_icon::create_shell_icon;
use winapi::um::winuser::{DispatchMessageA, TranslateMessage, GetMessageW};
use winapi::shared::ntdef::NULL;
use winapi::shared::windef::{HWND};

fn main() {


    create_shell_icon();
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


    // unsafe {
    //     if AllocConsole() != TRUE {panic!("Error opening console")};
    //     if SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) != TRUE {panic!("Error setting handler")};
    // }

    unsafe {
        let mut msg = std::mem::MaybeUninit::uninit().assume_init();
        loop {
            let ret = GetMessageW(&mut msg, NULL as HWND, 0, 0);
            match ret {
                -1 => { panic!("GetMessage failed"); }
                0 => { break }
                _ => {
                    TranslateMessage(&mut msg);
                    DispatchMessageA(&mut msg);
                }
            }
        }
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


