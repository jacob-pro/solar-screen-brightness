use crate::config::Config;
use crate::cursive::align::HAlign;
use crate::cursive::traits::Nameable;
use crate::cursive::view::Resizable;
use crate::cursive::views::{Dialog, HideableView, NamedView, ScrollView, SelectView, TextView};
use crate::cursive::Cursive;
use crate::tui::UserData;
use crate::APP_NAME;
use enum_iterator::IntoEnumIterator;

const MAIN_VIEW: &str = "MAIN_MENU_VIEW";
const MAIN_SELECT: &str = "MAIN_MENU_SELECT";

#[derive(IntoEnumIterator)]
pub enum MainMenuChoice {
    ShowStatus = 0,
    EditConfig,
    ReloadConfig,
    ToggleRunning,
    CloseConsole,
    ExitApplication,
}

impl MainMenuChoice {
    fn title(&self) -> &'static str {
        match self {
            MainMenuChoice::ShowStatus => "Show status",
            MainMenuChoice::EditConfig => "Edit configuration",
            MainMenuChoice::ReloadConfig => "Reload configuration",
            MainMenuChoice::ToggleRunning => "null",
            MainMenuChoice::CloseConsole => "Close console",
            MainMenuChoice::ExitApplication => "Exit application",
        }
    }
}

pub fn create() -> NamedView<HideableView<Dialog>> {
    let mut select = SelectView::new().h_align(HAlign::Left).autojump();
    for (idx, item) in MainMenuChoice::into_enum_iter().enumerate() {
        select.add_item(format!("{} {}", idx + 1, item.title()), item);
    }
    select.set_on_submit(on_submit);
    HideableView::new(
        Dialog::around(ScrollView::new(select.with_name(MAIN_SELECT))).title(APP_NAME),
    )
    .with_name(MAIN_VIEW)
}

pub fn running_change(s: &mut Cursive, running: bool) {
    s.call_on_name(MAIN_SELECT, |view: &mut SelectView<MainMenuChoice>| {
        let idx = MainMenuChoice::ToggleRunning as u8 as usize;
        let label = format!(
            "{} {} dynamic brightness",
            idx + 1,
            if running { "Disable" } else { "Enable" }
        );
        view.insert_item(idx, label, MainMenuChoice::ToggleRunning);
        view.remove_item(idx + 1);
    });
}

fn on_submit(cursive: &mut Cursive, choice: &MainMenuChoice) {
    let ud = cursive.user_data::<UserData>().unwrap();
    match choice {
        MainMenuChoice::ShowStatus => {
            let update = ud.controller.get_last_result();
            cursive
                .call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| {
                    x.hide();
                })
                .unwrap();
            let view = super::show_status::create(|x: &mut Cursive| {
                x.pop_layer();
                x.call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| {
                    x.unhide();
                })
                .unwrap();
            });
            cursive.add_layer(view);
            super::show_status::status_update(cursive, update);
        }
        MainMenuChoice::EditConfig => {
            let config = ud.controller.get_config();
            cursive
                .call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| {
                    x.hide();
                })
                .unwrap();
            let view = super::edit_config::create(config, |x: &mut Cursive| {
                x.pop_layer();
                x.call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| {
                    x.unhide();
                })
                .unwrap();
            });
            cursive.add_layer(view)
        }
        MainMenuChoice::ToggleRunning => {
            let enabled = ud.controller.get_enabled();
            ud.controller.set_enabled(!enabled);
        }
        MainMenuChoice::ReloadConfig => {
            let msg = match Config::load() {
                Ok(c) => {
                    ud.controller.set_config(c);
                    "Successfully loaded from disk".to_owned()
                }
                Err(e) => e.to_string(),
            };
            cursive.add_layer(Dialog::info(msg));
        }
        MainMenuChoice::CloseConsole => {
            let tray = ud.tray.clone();
            tray.close_console(cursive);
        }
        MainMenuChoice::ExitApplication => {
            let msg = "Warning: Exiting the application will stop the dynamic brightness \
            controller from running.\n\nYou may want to consider either closing the console window \
            or temporarily disabling dynamic brightness?";
            cursive.add_layer(
                Dialog::new()
                    .title("Confirm?")
                    .content(TextView::new(msg))
                    .dismiss_button("Cancel")
                    .button("Exit", |cursive| {
                        cursive.quit();
                        let ud = cursive.user_data::<UserData>().unwrap();
                        ud.tray.exit_application();
                    })
                    .max_width(40),
            );
        }
    }
}
