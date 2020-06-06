use cursive::Cursive;
use cursive::views::{SelectView, ScrollView, NamedView, Dialog, HideableView};
use cursive::align::HAlign;
use enum_iterator::IntoEnumIterator;
use crate::tui::UserData;
use crate::tray::TrayMessage;
use cursive::traits::Nameable;
use crate::brightness::BrightnessMessage;
use crate::config::Config;

const MAIN_VIEW: &str = "MAIN_MENU_VIEW";
const MAIN_SELECT: &str = "MAIN_MENU_SELECT";

#[derive(IntoEnumIterator)]
pub enum MainMenuChoice {
    ShowStatus = 0,
    EditConfig,
    SaveConfig,
    ReloadConfig,
    ToggleRunning,
    CloseConsole,
    ExitApplication,
}

impl MainMenuChoice {
    fn title(&self) -> &'static str {
        match self {
            MainMenuChoice::ShowStatus => {"Show status"},
            MainMenuChoice::EditConfig => {"Edit configuration"},
            MainMenuChoice::SaveConfig => {"Save configuration"},
            MainMenuChoice::ReloadConfig => {"Reload configuration"},
            MainMenuChoice::ToggleRunning => {"null"},
            MainMenuChoice::CloseConsole => {"Close console"},
            MainMenuChoice::ExitApplication => {"Exit Application"},
        }
    }
}

pub fn create() -> NamedView<HideableView<Dialog>> {
    let mut select = SelectView::new()
        .h_align(HAlign::Left)
        .autojump();
    for (idx, item) in MainMenuChoice::into_enum_iter().enumerate() {
        select.add_item(format!("{} {}", idx + 1, item.title()), item);
    }
    select.set_on_submit(on_submit);
    HideableView::new(Dialog::around(
        ScrollView::new(select.with_name(MAIN_SELECT))
    ).title("Solar Screen Brightness")).with_name(MAIN_VIEW)
}

pub fn running_change(s: &mut Cursive, running: bool) {
    s.call_on_name(MAIN_SELECT, |view: &mut SelectView<MainMenuChoice>| {
        let idx = MainMenuChoice::ToggleRunning as u8 as usize;
        let label = format!("{} {} dynamic brightness", idx + 1, if running { "Disable" } else { "Enable"});
        view.insert_item(idx, label, MainMenuChoice::ToggleRunning);
        view.remove_item(idx + 1);
    });
}

fn on_submit(cursive: &mut Cursive, choice: &MainMenuChoice) {
    let ud = cursive.user_data::<UserData>().unwrap();
    match choice {
        MainMenuChoice::ShowStatus => {
            let update = ud.status.read().unwrap().last_calculation().clone().unwrap();
            cursive.call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| { x.hide(); }).unwrap();
            let view = super::show_status::create(|x: &mut Cursive| {
                x.pop_layer();
                x.call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| { x.unhide(); }).unwrap();
            });
            cursive.add_layer(view);
            super::show_status::status_update(cursive, update);
        },
        MainMenuChoice::EditConfig => {
            let config = ud.status.read().unwrap().config.clone();
            cursive.call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| { x.hide(); }).unwrap();
            let view = super::edit_config::create(config, |x: &mut Cursive| {
                x.pop_layer();
                x.call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| { x.unhide(); }).unwrap();
            });
            cursive.add_layer(view)
        },
        MainMenuChoice::ToggleRunning => {
            let running = ud.status.read().unwrap().is_enabled();
            if running {
                ud.brightness.send(BrightnessMessage::Disable).unwrap();
            } else {
                ud.brightness.send(BrightnessMessage::Enable).unwrap();
            }
        }
        MainMenuChoice::SaveConfig => {
            let config = ud.status.read().unwrap().config.clone();
            let msg = match config.save() {
                Ok(_) => {"Successfully saved to disk".to_owned()},
                Err(e) => {e.to_string()},
            };
            cursive.add_layer(Dialog::info(msg));
        },
        MainMenuChoice::ReloadConfig => {
            let msg = match Config::load() {
                Ok(c) => {
                    ud.status.write().unwrap().config = c;
                    ud.brightness.send(BrightnessMessage::NewConfig).unwrap();
                    "Successfully loaded from disk".to_owned()
                },
                Err(e) => {e},
            };
            cursive.add_layer(Dialog::info(msg));
        }
        MainMenuChoice::CloseConsole => {
            &(ud.tray)(TrayMessage::CloseConsole);
        },
        MainMenuChoice::ExitApplication => {
            &(ud.tray)(TrayMessage::ExitApplication);
        },
    }
}
