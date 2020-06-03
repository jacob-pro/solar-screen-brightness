use cursive::Cursive;
use cursive::views::{SelectView, ScrollView, NamedView};
use cursive::align::HAlign;
use enum_iterator::IntoEnumIterator;
use crate::tui::UserData;
use crate::tray::TrayMessage;
use cursive::traits::Nameable;
use crate::brightness::BrightnessLoopMessage;


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

pub fn create() -> ScrollView<NamedView<SelectView<MainMenuChoice>>> {
    let mut select = SelectView::new()
        .h_align(HAlign::Left)
        .autojump();
    for (idx, item) in MainMenuChoice::into_enum_iter().enumerate() {
        select.add_item(format!("{} {}", idx + 1, item.title()), item);
    }
    select.set_on_submit(on_submit);
    ScrollView::new(select.with_name("MainSelect"))
}

pub fn running_change(s: &mut Cursive, running: bool) {
    s.call_on_name("MainSelect", |view: &mut SelectView<MainMenuChoice>| {
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

        },
        MainMenuChoice::EditConfig => {

        },
        MainMenuChoice::ToggleRunning => {
            let running = *ud.status.read().unwrap().running();
            if running {
                ud.brightness.send(BrightnessLoopMessage::Pause).unwrap();
            } else {
                ud.brightness.send(BrightnessLoopMessage::Resume).unwrap();
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
