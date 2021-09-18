mod assets;
mod install;
mod uninstall;

use clap::{AppSettings, Clap};
use directories::BaseDirs;
use env_logger::Env;
use lazy_static::lazy_static;
use std::env::consts::EXE_EXTENSION;
use std::path::PathBuf;

lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = {
        let p = BaseDirs::new()
            .unwrap()
            .config_dir()
            .join("Solar Screen Brightness");
        log::trace!("Ensuring {:?} folder exists", p);
        std::fs::create_dir_all(&p).unwrap();
        p
    };
    pub static ref BINARY_PATH: PathBuf = CONFIG_DIR
        .join("solar-screen-brightness")
        .with_extension(EXE_EXTENSION);
}

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
            SubCommand::Tui => install::install(),
            SubCommand::Install => install::install(),
            SubCommand::Uninstall => install::install(),
        } {
            log::error!("{:#}", e);
            return 1;
        }
        0
    })());
}
