#![windows_subsystem = "windows"]

#[macro_use]
extern crate validator_derive;

mod assets;
mod brightness;
mod config;
mod console;
mod controller;
mod tray;
mod tui;
#[cfg(target_os = "windows")]
mod wide;

#[cfg(not(target_os = "windows"))]
pub use cursive;
#[cfg(target_os = "windows")]
pub use solar_screen_brightness_windows_bindings::cursive;

use crate::config::Config;
use crate::controller::apply::get_devices;
use crate::controller::BrightnessController;
use clap::{AppSettings, Clap};
use futures::executor::block_on;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

// use crate::wide::WideString;

// use solar_screen_brightness_windows_bindings::Windows::Win32::{
//     Foundation::{BOOL, HWND, PWSTR},
//     System::Diagnostics::Debug::{GetLastError, SetLastError, WIN32_ERROR},
//     System::Threading::CreateMutexW,
// };

#[derive(Clap)]
#[clap(version = "1.0", author = "Jacob Halsey <jacob@jhalsey.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(Clap, Default)]
struct LaunchArgs {
    #[clap(long, about = "Don't automatically display the console")]
    hide_console: bool,
}

#[derive(Clap)]
struct HeadlessArgs {
    #[clap(long, about = "Compute and apply brightness once, then exit")]
    once: bool,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "(default)")]
    Launch(LaunchArgs),
    #[clap(about = "Runs dynamic brightness without a tray / GUI")]
    Headless(HeadlessArgs),
    #[clap(about = "Lists detected monitors")]
    ListMonitors,
}

impl Default for SubCommand {
    fn default() -> Self {
        SubCommand::Launch(Default::default())
    }
}

fn main() {
    std::process::exit((|| {
        let _console = win32_subsystem_fix::run();
        let opts: Opts = match Opts::try_parse() {
            Err(e) => {
                e.print().ok();
                return EXIT_FAILURE;
            }
            Ok(s) => s,
        };
        match opts.subcmd.unwrap_or_default() {
            SubCommand::Launch(args) => launch(args),
            SubCommand::Headless(args) => headless(args),
            SubCommand::ListMonitors => list_monitors(),
        }
    })());
}

fn launch(args: LaunchArgs) -> i32 {
    env_logger::init();
    let config = Config::load().ok().unwrap_or_default();
    let mut controller = BrightnessController::new(config);
    controller.start();
    tray::run_tray_application(controller, !args.hide_console);
    log::info!("Program exiting gracefully");
    EXIT_SUCCESS
}

fn headless(args: HeadlessArgs) -> i32 {
    env_logger::init();
    let config = match Config::load() {
        Ok(c) => {
            if c.location.is_none() {
                log::error!("Config file loaded, but no location has been configured");
                return EXIT_FAILURE;
            }
            c
        }
        Err(_) => {
            return EXIT_FAILURE;
        }
    };
    if args.once {
        let (_res, wait) = controller::apply::apply(config, true);
        wait.map(|wait| log::info!("Brightness valid until: {}", wait));
    } else {
        let mut controller = BrightnessController::new(config);
        controller.start();
        loop {
            std::thread::park();
        }
    }
    log::info!("Program exiting gracefully");
    EXIT_SUCCESS
}

fn list_monitors() -> i32 {
    let devices = block_on(get_devices());
    println!("Detecting attached monitors:");
    println!("{} monitors", devices.len());
    EXIT_SUCCESS
}

// fn already_running() -> bool {
//     const ERROR_ALREADY_EXISTS: WIN32_ERROR = WIN32_ERROR(183);
//     unsafe {
//         let mut name = "solar-screen-brightness".to_wide();
//         SetLastError(0);
//         CreateMutexW(
//             std::ptr::null_mut(),
//             BOOL::from(true),
//             PWSTR(name.as_mut_ptr()),
//         );
//         return GetLastError() == ERROR_ALREADY_EXISTS;
//     }
// }

#[cfg(not(target_os = "windows"))]
pub(crate) mod win32_subsystem_fix {
    pub(super) fn run() {}
}

#[cfg(target_os = "windows")]
// https://www.tillett.info/2013/05/13/how-to-create-a-windows-program-that-works-as-both-as-a-gui-and-console-application/
pub mod win32_subsystem_fix {
    use solar_screen_brightness_windows_bindings::Windows::Win32::{
        System::Console::*, UI::KeyboardAndMouseInput::*,
        UI::WindowsAndMessaging::SetForegroundWindow,
    };

    pub(super) struct ConsoleAttachment();

    impl Drop for ConsoleAttachment {
        fn drop(&mut self) {
            send_enter();
        }
    }

    pub(super) fn run() -> Option<ConsoleAttachment> {
        const ATTACH_PARENT_PROCESS: u32 = -1i32 as u32;
        unsafe {
            let attached = AttachConsole(ATTACH_PARENT_PROCESS).as_bool();
            if attached {
                println!();
                return Some(ConsoleAttachment());
            }
        }
        None
    }

    // Call this on an early/unexpected process exit
    // Otherwise will automatically be called if the main function returns
    pub fn send_enter() {
        unsafe {
            let console = GetConsoleWindow();
            if !console.is_null() && SetForegroundWindow(console).as_bool() {
                let mut ip: INPUT = std::mem::MaybeUninit::zeroed().assume_init();
                ip.r#type = INPUT_KEYBOARD;
                ip.Anonymous.ki.wVk = 0x0D; // virtual-key code for the "Enter" key
                SendInput(1, &mut ip, std::mem::size_of_val(&ip) as i32);
                ip.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
                SendInput(1, &mut ip, std::mem::size_of_val(&ip) as i32);
            }
        }
    }
}
