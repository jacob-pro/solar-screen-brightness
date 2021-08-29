use crate::controller::BrightnessController;
use crate::tray::TrayApplicationHandle;
use crate::tui::launch_cursive;

pub(super) struct Console {
    tray: TrayApplicationHandle,
    controller: BrightnessController,
    running: bool,
}

impl Console {
    pub(super) fn new(tray: TrayApplicationHandle, controller: BrightnessController) -> Self {
        Self {
            tray,
            controller,
            running: false,
        }
    }

    pub(super) fn show(&mut self) {
        if !self.running {
            let tray = self.tray.clone();
            let controller = self.controller.clone();
            launch_cursive(tray, controller);
            self.running = true;
        }
    }

    pub(super) fn hide(&mut self) {
        self.running = false;
    }
}
