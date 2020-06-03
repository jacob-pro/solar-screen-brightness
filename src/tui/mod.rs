use cursive::views::{Dialog};
use cursive::{Cursive, CursiveExt, CbSink};
use cursive::event::Event;
use crate::tray::TrayMessageSender;
use crate::brightness::{BrightnessMessageSender, BrightnessStatusRef, BrightnessStatusDelegate};
use std::sync::Arc;

mod main_menu;

pub struct UserData {
    tray: TrayMessageSender,
    brightness: BrightnessMessageSender,
    status: BrightnessStatusRef,
}

struct Delegate(CbSink);
impl BrightnessStatusDelegate for Delegate {
    fn on_toggle(&self, running: bool) {
        self.0.send(Box::new(move |s| {
            main_menu::running_change(s, running);
        })).unwrap();
    }
}

pub fn run(tray: TrayMessageSender, brightness: BrightnessMessageSender, status: BrightnessStatusRef) {
    let mut siv = Cursive::crossterm().unwrap();

    siv.clear_global_callbacks(Event::CtrlChar('c'));
    siv.clear_global_callbacks(Event::Exit);

    siv.set_user_data(UserData {
        tray,
        brightness,
        status: status.clone()
    });

    let menu = main_menu::create();
    siv.add_layer(Dialog::around(menu).title("Solar Screen Brightness"));
    main_menu::running_change(&mut siv, *status.read().unwrap().running());

    let delegate: Arc<Box<dyn BrightnessStatusDelegate + Send + Sync>> = Arc::new(Box::new(Delegate(siv.cb_sink().clone())));
    status.write().unwrap().delegate = Arc::downgrade(&delegate);

    siv.run();
}
