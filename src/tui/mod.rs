use cursive::views::Dialog;
use cursive::{Cursive, CursiveExt};
use cursive::event::Event;
use crate::tray::TrayMessageSender;
use crate::brightness::{BrightnessMessageSender, BrightnessStatusRef};

mod main_menu;

pub struct UserData {
    tray: TrayMessageSender,
    brightness: BrightnessMessageSender,
    status: BrightnessStatusRef,
}

pub fn run(tray: TrayMessageSender, brightness: BrightnessMessageSender, status: BrightnessStatusRef) {
    let mut siv = Cursive::crossterm().unwrap();

    siv.clear_global_callbacks(Event::CtrlChar('c'));
    siv.clear_global_callbacks(Event::Exit);

    siv.set_user_data(UserData {
        tray,
        brightness,
        status
    });

    siv.add_layer(
        Dialog::around(main_menu::create())
            .title("Solar Screen Brightness")
    );

    siv.run();
}
