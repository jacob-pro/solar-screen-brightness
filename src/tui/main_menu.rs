use cursive::Cursive;
use cursive::views::{SelectView, ScrollView, NamedView, Dialog, HideableView};
use cursive::align::HAlign;
use enum_iterator::IntoEnumIterator;
use crate::tui::UserData;
use crate::tray::TrayMessage;
use cursive::traits::Nameable;
use crate::brightness::BrightnessMessage;

const MAIN_VIEW: &str = "MainMenu";
const MAIN_SELECT: &str = "MainSelect";

#[derive(IntoEnumIterator)]
pub enum MainMenuChoice {
    ShowStatus = 0,
    EditConfig,
    SaveConfig,
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
            let update = ud.status.read().unwrap().last_update().clone().unwrap();
            cursive.call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| { x.hide(); }).unwrap();
            let view = super::status::create(|x| {
                x.pop_layer();
                x.call_on_name(MAIN_VIEW, |x: &mut HideableView<Dialog>| { x.unhide(); }).unwrap();
            });
            cursive.add_layer(view);
            super::status::status_update(cursive, update);
        },
        MainMenuChoice::EditConfig => {

        },
        MainMenuChoice::ToggleRunning => {
            let running = *ud.status.read().unwrap().running();
            if running {
                ud.brightness.send(BrightnessMessage::Pause).unwrap();
            } else {
                ud.brightness.send(BrightnessMessage::Resume).unwrap();
            }
        }
        MainMenuChoice::SaveConfig => {
            let status = ud.status.read().unwrap();
            let running = status.config().clone();
            running.save();
        },
        MainMenuChoice::CloseConsole => {
            &(ud.tray)(TrayMessage::CloseConsole);
        },
        MainMenuChoice::ExitApplication => {
            &(ud.tray)(TrayMessage::ExitApplication);
        },
    }

}
