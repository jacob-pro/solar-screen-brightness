mod assets;
mod install;
mod tui;
mod uninstall;

use clap::{AppSettings, Clap};
use env_logger::Env;

#[cfg(not(windows))]
pub use cursive;
#[cfg(windows)]
pub use solar_screen_brightness_windows::cursive;

pub const APP_NAME: &'static str = "Solar Screen Brightness";

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Opens the TUI window (default)")]
    Tui,
    #[clap(about = "Installs solar screen brightness for the current user")]
    Install,
    #[clap(about = "Uninstalls solar screen brightness for the current user")]
    Uninstall,
}

impl Default for SubCommand {
    fn default() -> Self {
        Self::Tui
    }
}

#[derive(Clap)]
#[clap(version = "1.0", author = "Jacob Halsey <jacob@jhalsey.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    sub_command: Option<SubCommand>,
}

fn main() {
    let opts: Opts = Opts::parse();
    std::process::exit((|| {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        if let Err(e) = match opts.sub_command.unwrap_or_default() {
            SubCommand::Tui => tui::launch_cursive(),
            SubCommand::Install => install::install().and_then(|_| install::launch()),
            SubCommand::Uninstall => uninstall::uninstall(),
        } {
            log::error!("{:#}", e);
            return 1;
        }
        0
    })());
}
