//! Entry point for CLI driven application
use anyhow::Context;
use clap::Parser;
use solar_screen_brightness::apply::apply_brightness;
use solar_screen_brightness::common::{install_logger, APP_NAME};
use solar_screen_brightness::config::SsbConfig;
use solar_screen_brightness::controller::BrightnessController;
use solar_screen_brightness::event_watcher::EventWatcher;
use solar_screen_brightness::unique::SsbUniqueInstance;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, RwLock};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    /// Update brightness once and exit
    #[arg(short, long)]
    once: bool,
    /// Override the config file path
    #[arg(long)]
    config: Option<PathBuf>,
}

fn run(args: Args) -> anyhow::Result<()> {
    log::info!(
        "Starting {} (CLI), version: {}",
        APP_NAME,
        env!("CARGO_PKG_VERSION")
    );
    let config = SsbConfig::load(args.config)
        .context("Unable to load config file")?
        .context("Config file does not exist")?;
    config
        .location
        .as_ref()
        .context("Location is not configured")?;
    if args.once {
        let result = apply_brightness(
            config.brightness_day,
            config.brightness_night,
            config.transition_mins,
            config.location.unwrap(),
            config.overrides,
        );
        let pretty = serde_json::to_string_pretty(&result).unwrap();
        println!("{}", pretty);
    } else {
        let (tx, rx) = mpsc::channel();
        let config = Arc::new(RwLock::new(config));
        let controller = BrightnessController::start(config, || {});
        let _event_watcher = EventWatcher::start(&controller, None);
        ctrlc::set_handler(move || tx.send(()).unwrap()).expect("Error setting Ctrl-C handler");
        rx.recv().expect("Could not receive from channel.");
    }
    Ok(())
}

fn main() {
    let args: Args = Args::parse();

    // Check this is the only instance running
    let _unique_instance = match SsbUniqueInstance::try_acquire() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Setup logging
    if let Err(e) = install_logger(args.debug, false) {
        eprintln!("Unable to install logger: {:#}", e);
        std::process::exit(1);
    }
    // Run the application logic
    if let Err(e) = run(args) {
        log::error!("{:#}", e);
        std::process::exit(1);
    }
}
