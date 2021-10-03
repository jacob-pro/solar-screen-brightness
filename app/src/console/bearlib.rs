use crate::controller::BrightnessController;
use crate::tray::TrayApplicationHandle;
use crate::tui::launch_cursive;
use std::sync::Arc;

pub(super) struct Console {
    tray: TrayApplicationHandle,
    controller: Arc<BrightnessController>,
    running: bool,
}

impl Console {
    pub(super) fn new(tray: TrayApplicationHandle, controller: Arc<BrightnessController>) -> Self {
        Self {
            tray,
            controller,
            running: false,
        }
    }

    pub(super) fn show(&mut self) {
        if !self.running {
            let tray = self.tray.clone();
            launch_cursive(tray, Arc::clone(&self.controller));
            self.running = true;
        }
    }

    pub(super) fn hide(&mut self) {
        self.running = false;
    }
}
