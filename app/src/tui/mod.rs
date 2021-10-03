use crate::controller::apply::ApplyResult;
use crate::controller::{BrightnessController, Delegate};
use crate::cursive::event::Event;
use crate::cursive::{CbSink, Cursive, CursiveExt};
use crate::tray::TrayApplicationHandle;
use std::sync::Arc;

mod edit_config;
mod main_menu;
mod show_status;

pub struct UserData {
    tray: TrayApplicationHandle,
    controller: Arc<BrightnessController>,
}

struct CursiveObserver(CbSink);

impl Delegate for CursiveObserver {
    fn did_set_enabled(&self, running: bool) {
        self.0
            .send(Box::new(move |s| {
                main_menu::running_change(s, running);
            }))
            .unwrap();
    }
    fn did_set_last_result(&self, update: &ApplyResult) {
        let update = update.clone();
        self.0
            .send(Box::new(move |s| {
                show_status::status_update(s, update);
            }))
            .unwrap();
    }
}

pub fn launch_cursive(tray: TrayApplicationHandle, controller: Arc<BrightnessController>) {
    std::thread::spawn(move || {
        log::info!("Cursive thread starting");
        let mut siv = Cursive::default();

        siv.clear_global_callbacks(Event::CtrlChar('c'));
        siv.clear_global_callbacks(Event::Exit);

        siv.set_user_data(UserData {
            tray,
            controller: Arc::clone(&controller),
        });

        siv.add_layer(main_menu::create());
        main_menu::running_change(&mut siv, controller.get_enabled());

        let delegate: Arc<dyn Delegate + Send + Sync> =
            Arc::new(CursiveObserver(siv.cb_sink().clone()));

        controller.set_delegate(Arc::downgrade(&delegate));

        siv.run();
        log::info!("Cursive thread stopping");
    });
}
