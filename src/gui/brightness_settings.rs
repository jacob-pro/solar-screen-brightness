use crate::config::SsbConfig;
use crate::controller::Message;
use crate::gui::app::{AppState, MessageModal, Page, SPACING};
use validator::Validate;

pub struct BrightnessSettingsPage {
    brightness_day: u32,
    brightness_night: u32,
    transition_mins: u32,
}

impl BrightnessSettingsPage {
    pub fn from_config(config: &SsbConfig) -> Self {
        Self {
            brightness_day: config.brightness_day,
            brightness_night: config.brightness_night,
            transition_mins: config.transition_mins,
        }
    }

    fn copy_to_config(&self, config: &mut SsbConfig) {
        config.brightness_night = self.brightness_night;
        config.brightness_day = self.brightness_day;
        config.transition_mins = self.transition_mins;
        assert!(config.validate().is_ok())
    }
}

impl Page for BrightnessSettingsPage {
    fn render(&mut self, ui: &mut egui::Ui, app_state: &mut AppState) {
        egui::Grid::new("brightness_settings")
            .num_columns(2)
            .show(ui, |ui| {

                ui.label("Day Brightness");
                ui.add(egui::Slider::new(&mut self.brightness_day, 0u32..=100u32).suffix("%"));
                ui.end_row();

                ui.label("Night Brightness");
                ui.add(egui::Slider::new(&mut self.brightness_night, 0u32..=100u32).suffix("%"));
                ui.end_row();

                ui.label("Transition Minutes").on_hover_text("How long it takes to transition between day and night brightness at sunset/sunrise");
                ui.add(egui::Slider::new(&mut self.transition_mins, 0u32..=360u32).suffix("min"));
                ui.end_row();

            });
        ui.add_space(SPACING);
        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
            if ui.button("Apply").clicked() {
                let mut config = app_state.config.write().unwrap();
                self.copy_to_config(&mut config);
                app_state
                    .controller
                    .send(Message::Refresh("Setting change"))
                    .unwrap();
            }
            if ui.button("Save").clicked() {
                let mut config = app_state.config.write().unwrap();
                self.copy_to_config(&mut config);
                app_state
                    .controller
                    .send(Message::Refresh("Setting change"))
                    .unwrap();
                if let Err(e) = config.save() {
                    log::error!("Unable to save config: {:#}", e);
                    app_state.transitions.queue_state_transition(move |app| {
                        app.modal = Some(Box::new(MessageModal {
                            title: "Error".to_string(),
                            message: format!("Unable to save config: {}", e),
                        }));
                    });
                }
            };
        });
    }
}
