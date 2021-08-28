use crate::controller::BrightnessController;
use crate::tray::TrayApplicationHandle;
use crate::tui::run_cursive;

pub(super) struct Console {
    tray: TrayApplicationHandle,
    controller: BrightnessController,
}

impl Console {
    pub(super) fn new(tray: TrayApplicationHandle, controller: BrightnessController) -> Self {
        Self { tray, controller }
    }

    pub(super) fn show(&mut self) {
        let tray = self.tray.clone();
        let controller = self.controller.clone();
        std::thread::spawn(move || {
            run_cursive(tray, controller);
        });
    }

    pub(super) fn hide(&self) {}
}
