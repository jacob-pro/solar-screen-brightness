#[macro_use]
extern crate validator_derive;
#[macro_use]
extern crate maplit;

mod assets;
mod brightness;
mod config;
mod console;
mod controller;
mod lock;
mod tray;
mod tui;
#[cfg(windows)]
mod wide;

#[cfg(not(windows))]
pub use cursive;
#[cfg(windows)]
pub use solar_screen_brightness_windows_bindings::cursive;

use crate::config::Config;
use crate::controller::apply::{get_devices, get_properties};
use crate::controller::BrightnessController;
use crate::lock::ApplicationLock;
use crate::tray::show_console_in_another_process;
use clap::{AppSettings, Clap};
use futures::executor::block_on;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

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
        console_subsystem_fix();
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
    match ApplicationLock::acquire() {
        Some(_lock) => {
            let config = Config::load().ok().unwrap_or_default();
            let mut controller = BrightnessController::new(config);
            controller.start();
            tray::run_tray_application(controller, !args.hide_console);
            log::info!("Program exiting gracefully");
            EXIT_SUCCESS
        }
        None => {
            log::error!("Failed to acquire lock - the application is already running");
            if !args.hide_console {
                show_console_in_another_process();
            }
            EXIT_FAILURE
        }
    }
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
        match ApplicationLock::acquire() {
            Some(_lock) => {
                let mut controller = BrightnessController::new(config);
                controller.start();
                loop {
                    std::thread::park();
                }
            }
            None => {
                log::error!("Failed to acquire lock - the application is already running");
                return EXIT_FAILURE;
            }
        }
    }
    log::info!("Program exiting gracefully");
    EXIT_SUCCESS
}

fn list_monitors() -> i32 {
    let devices = block_on(get_devices());
    for r in &devices {
        match r {
            Ok(device) => match block_on(get_properties(device)) {
                Ok(info) => {
                    println!();
                    let mut keys = info.keys().collect::<Vec<_>>();
                    keys.sort();
                    for k in keys {
                        println!("{}: \"{}\"", k, info.get(k).unwrap());
                    }
                }
                Err(e) => {
                    println!("\nFound unknown device:\n{}", e);
                }
            },
            Err(e) => {
                println!("\nFailed to load device:\n{}", e);
            }
        }
    }
    if devices.iter().find(|e| e.is_err()).is_some() {
        EXIT_FAILURE
    } else {
        EXIT_SUCCESS
    }
}

#[cfg(not(target_os = "windows"))]
pub fn console_subsystem_fix() {}

#[cfg(target_os = "windows")]
pub fn console_subsystem_fix() {
    use solar_screen_brightness_windows_bindings::Windows::Win32::{
        System::Console::GetConsoleWindow,
        System::Threading::GetCurrentProcessId,
        UI::WindowsAndMessaging::{GetWindowThreadProcessId, ShowWindow, SW_HIDE},
    };
    // This app is built with /SUBSYTEM:CONSOLE
    // This is so that we can use the console functions or view the logs
    // However when launched as a desktop application Windows auto starts a console window
    // in this process, so we need to hide it
    unsafe {
        let console = GetConsoleWindow();
        if !console.is_null() {
            let mut console_pid = 0;
            GetWindowThreadProcessId(console, &mut console_pid);
            if console_pid == GetCurrentProcessId() {
                log::info!("Hiding Windows console subsystem window");
                ShowWindow(console, SW_HIDE);
            }
        }
    }
}
