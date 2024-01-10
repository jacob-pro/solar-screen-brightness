#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use clap::Parser;
use egui_winit::winit;
use solar_screen_brightness::common::{install_logger, APP_NAME};
use solar_screen_brightness::config::SsbConfig;
use solar_screen_brightness::controller::BrightnessController;
use solar_screen_brightness::event_watcher::EventWatcher;
use solar_screen_brightness::gui::app::SsbEguiApp;
use solar_screen_brightness::gui::{NextPaint, UserEvent, WgpuWinitApp};
use solar_screen_brightness::unique::SsbUniqueInstance;
use solar_screen_brightness::{tray, unique};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use winit::event::Event;
use winit::event_loop::EventLoopBuilder;
use winit::platform::run_return::EventLoopExtRunReturn;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    /// Start minimised
    #[arg(short, long)]
    minimised: bool,
}

fn main() {
    set_panic_hook();

    let args: Args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => handle_invalid_args(e),
    };

    // Check this is the only instance running
    let _unique_instance = match SsbUniqueInstance::try_acquire() {
        Ok(i) => i,
        Err(unique::Error::PlatformError(e)) => {
            panic!("Unexpected platform error: {:#}", e);
        }
        Err(unique::Error::AlreadyRunning(i)) => {
            eprintln!("{} is already running, attempting to wakeup", APP_NAME);
            i.wakeup();
            std::process::exit(1);
        }
    };

    // Setup logging
    if let Err(e) = install_logger(args.debug, true) {
        panic!("Unable to install logger: {:#}", e);
    }

    log::info!(
        "Starting {} (GUI), version: {}",
        APP_NAME,
        env!("CARGO_PKG_VERSION")
    );

    let config = Arc::new(RwLock::new(
        SsbConfig::load(None)
            .expect("Unable to load config")
            .unwrap_or_default(),
    ));

    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let controller_proxy = event_loop.create_proxy();
    let controller = BrightnessController::start(config.clone(), move || {
        controller_proxy
            .send_event(UserEvent::RepaintNow("Brightness Controller Update"))
            .unwrap();
    });

    let ctrlc_proxy = event_loop.create_proxy();
    ctrlc::set_handler(move || ctrlc_proxy.send_event(UserEvent::Exit("ctrl-c")).unwrap()).unwrap();

    let _event_watcher = EventWatcher::start(&controller, Some(&event_loop));
    let _tray = tray::create(&event_loop);

    let app_proxy = event_loop.create_proxy();
    let mut framework = WgpuWinitApp::new(event_loop.create_proxy(), args.minimised, move || {
        SsbEguiApp::new(
            app_proxy.clone(),
            controller.last_result.clone(),
            config.clone(),
            controller.sender.clone(),
        )
    });

    let mut next_repaint_time = Some(Instant::now());

    event_loop.run_return(|event, event_loop, control_flow| {
        let event_result = match &event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            #[cfg(target_os = "windows")]
            Event::RedrawEventsCleared => {
                next_repaint_time = None;
                framework.paint()
            }
            #[cfg(not(target_os = "windows"))]
            Event::RedrawRequested(_) => {
                next_repaint_time = None;
                framework.paint()
            }
            event => framework.on_event(event_loop, event),
        };

        match event_result {
            NextPaint::Wait => {}
            NextPaint::RepaintNext => {
                next_repaint_time = Some(Instant::now());
            }
            NextPaint::RepaintAt(repaint_time) => {
                next_repaint_time =
                    Some(next_repaint_time.unwrap_or(repaint_time).min(repaint_time));
            }
            NextPaint::Exit => {
                control_flow.set_exit();
                return;
            }
        }

        if let Some(time) = next_repaint_time {
            if time <= Instant::now() {
                if let Some(window) = framework.window() {
                    window.request_redraw();
                }
                next_repaint_time = None;
                control_flow.set_poll();
            } else {
                control_flow.set_wait_until(time);
            };
        } else {
            control_flow.set_wait();
        }
    });
}

#[cfg(not(windows))]
fn handle_invalid_args(err: clap::Error) -> ! {
    err.exit()
}

#[cfg(windows)]
fn handle_invalid_args(err: clap::Error) -> ! {
    use windows::Win32::System::Console::AllocConsole;
    let allocated = unsafe { AllocConsole().ok().is_some() };
    err.print().unwrap();
    if allocated {
        println!("\nPress any key to close...");
        console::Term::stdout().read_key().unwrap();
    }
    std::process::exit(2);
}

#[cfg(not(windows))]
fn set_panic_hook() {
    std::panic::set_hook(Box::new(move |info| {
        log::error!("Fatal error: {}", info);
        std::process::exit(1);
    }));
}

#[cfg(windows)]
fn set_panic_hook() {
    use win32_utils::str::ToWin32Str;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::MessageBoxW;
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONSTOP, MB_OK};

    std::panic::set_hook(Box::new(move |info| unsafe {
        log::error!("Fatal error: {}", info);
        let title = "Fatal Error".to_wchar();
        let text = format!("{}", info).to_wchar();
        MessageBoxW(
            HWND::default(),
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONSTOP,
        );
        std::process::exit(1);
    }));
}
