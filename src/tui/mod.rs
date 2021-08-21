use crate::config::Config;
use crate::controller::apply::ApplyResult;
use crate::controller::state::Observer;
use crate::controller::StateRef;
use crate::tray::TrayMessageSender;
use cursive::event::Event;
use cursive::{CbSink, Cursive, CursiveExt};
use std::sync::Arc;

mod edit_config;
mod main_menu;
mod show_status;

pub struct UserData {
    tray: TrayMessageSender,
    state: StateRef,
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

pub fn run(tray: TrayMessageSender, state: StateRef) {
    let mut siv = Cursive::crossterm().unwrap();

    siv.clear_global_callbacks(Event::CtrlChar('c'));
    siv.clear_global_callbacks(Event::Exit);

    siv.set_user_data(UserData {
        tray,
        state: state.clone(),
    });

    siv.add_layer(main_menu::create());
    main_menu::running_change(&mut siv, state.read().unwrap().get_enabled());

    let delegate: Arc<dyn Observer + Send + Sync> =
        Arc::new(CursiveObserver(siv.cb_sink().clone()));

    state.write().unwrap().register(Arc::downgrade(&delegate));

    siv.run();
}
