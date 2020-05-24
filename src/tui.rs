use cursive::views::{Dialog, TextView};
use cursive::{Cursive, CursiveExt};
use cursive::event::Event;
use crate::tray::{MessageSender, TrayMessage};
use std::rc::Rc;

pub fn run_tui(tray: MessageSender) {
    let tray = Rc::new(tray);
    let mut siv = Cursive::crossterm().unwrap();
    siv.clear_global_callbacks(Event::CtrlChar('c'));
    siv.clear_global_callbacks(Event::Exit);

    // Creates a dialog with a single "Quit" button
    let tray1 = tray.clone();
    let tray2 = tray.clone();
    siv.add_layer(Dialog::around(
        TextView::new("Hello Dialog!"))
        .title("Cursive")
        .button("Close Console", move |_| {
            tray1(TrayMessage::CloseConsole);
        }).button("Exit", move |_| {
        tray2(TrayMessage::ExitApplication);
    }));

    // Starts the event loop.
    siv.run();
}
