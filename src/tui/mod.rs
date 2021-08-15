use crate::runner::{
    BrightnessMessageSender, BrightnessStatusDelegate, BrightnessStatusRef, LastCalculation,
};
use crate::tray::TrayMessageSender;
use cursive::event::Event;
use cursive::{CbSink, Cursive, CursiveExt};
use std::sync::Arc;

mod edit_config;
mod main_menu;
mod show_status;

pub struct UserData {
    tray: TrayMessageSender,
    brightness: BrightnessMessageSender,
    status: BrightnessStatusRef,
}

struct Delegate(CbSink);
impl BrightnessStatusDelegate for Delegate {
    fn running_change(&self, running: &bool) {
        let running = *running;
        self.0
            .send(Box::new(move |s| {
                main_menu::running_change(s, running);
            }))
            .unwrap();
    }
    fn update_change(&self, update: &LastCalculation) {
        let update = update.clone();
        self.0
            .send(Box::new(move |s| {
                show_status::status_update(s, update);
            }))
            .unwrap();
    }
}

pub fn run(
    tray: TrayMessageSender,
    brightness: BrightnessMessageSender,
    status: BrightnessStatusRef,
) {
    let mut siv = Cursive::crossterm().unwrap();

    siv.clear_global_callbacks(Event::CtrlChar('c'));
    siv.clear_global_callbacks(Event::Exit);

    siv.set_user_data(UserData {
        tray,
        brightness,
        status: status.clone(),
    });

    siv.add_layer(main_menu::create());
    main_menu::running_change(&mut siv, status.read().unwrap().is_enabled());

    let delegate: Arc<Box<dyn BrightnessStatusDelegate + Send + Sync>> =
        Arc::new(Box::new(Delegate(siv.cb_sink().clone())));
    status.write().unwrap().delegate = Arc::downgrade(&delegate);

    siv.run();
}
