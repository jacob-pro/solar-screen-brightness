use crate::config::Config;
use crate::controller::apply::ApplyResult;
use crate::controller::{BrightnessController, Observer};
use crate::cursive::event::Event;
use crate::cursive::{CbSink, Cursive, CursiveExt};
use crate::tray::TrayApplicationHandle;
use std::sync::Arc;

mod edit_config;
mod main_menu;
mod show_status;

pub struct UserData {
    tray: TrayApplicationHandle,
    controller: BrightnessController,
}

struct CursiveObserver(CbSink);

impl Observer for CursiveObserver {
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

    fn did_set_config(&self, _config: &Config) {}
}

pub fn launch_cursive(tray: TrayApplicationHandle, controller: BrightnessController) {
    std::thread::spawn(move || {
        log::info!("Cursive thread starting");
        let mut siv = Cursive::default();

        siv.clear_global_callbacks(Event::CtrlChar('c'));
        siv.clear_global_callbacks(Event::Exit);

        siv.set_user_data(UserData {
            tray,
            controller: controller.clone(),
        });

        siv.add_layer(main_menu::create());
        main_menu::running_change(&mut siv, controller.get_enabled());

        let delegate: Arc<dyn Observer + Send + Sync> =
            Arc::new(CursiveObserver(siv.cb_sink().clone()));

        controller.register(Arc::downgrade(&delegate));

        siv.run();
        log::info!("Cursive thread stopping");
    });
}
