use cursive::Cursive;
use cursive::views::{SelectView, ScrollView};
use cursive::align::HAlign;
use enum_iterator::IntoEnumIterator;
use crate::tui::UserData;
use crate::tray::TrayMessage;

#[derive(IntoEnumIterator)]
pub enum MainMenuChoice {
    ShowStatus,
    EditConfig,
    SaveConfig,
    CloseConsole,
    ExitApplication,
}

impl MainMenuChoice {

    fn title(&self) -> &'static str {
        match self {
            MainMenuChoice::ShowStatus => {"Show status"},
            MainMenuChoice::EditConfig => {"Edit configuration"},
            MainMenuChoice::SaveConfig => {"Save configuration"},
            MainMenuChoice::CloseConsole => {"Close console"},
            MainMenuChoice::ExitApplication => {"Exit Application"},
        }
    }

}

pub fn create() -> ScrollView<SelectView<MainMenuChoice>> {
    let mut select = SelectView::new()
        .h_align(HAlign::Left)
        .autojump();
    for (idx, item) in MainMenuChoice::into_enum_iter().enumerate() {
        select.add_item(format!("{} {}", idx + 1, item.title()), item);
    }
    select.set_on_submit(on_submit);
    ScrollView::new(select)
}

fn on_submit(cursive: &mut Cursive, choice: &MainMenuChoice) {
    let ud = cursive.user_data::<UserData>().unwrap();
    match choice {
        MainMenuChoice::ShowStatus => {

        },
        MainMenuChoice::EditConfig => {

        },
        MainMenuChoice::SaveConfig => {
            let status = ud.status.read().unwrap();
            let running = status.config.clone();
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
