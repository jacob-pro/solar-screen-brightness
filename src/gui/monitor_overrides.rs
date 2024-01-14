use crate::apply::ApplyResults;
use crate::config::{MonitorProperty, SsbConfig};
use crate::gui::app::{AppState, Page, SPACING};
use ellipse::Ellipse;

pub struct MonitorOverridePage {}

impl Page for MonitorOverridePage {
    fn render(&mut self, ui: &mut egui::Ui, context: &mut AppState) {
        let results = context.results.read().unwrap();
        if let Some(results) = results.as_ref() {
            ui.label(egui::RichText::new("Devices").size(14.0));
            ui.add_space(SPACING);
            self.render_monitors(ui, results);
            ui.add_space(SPACING);

            ui.separator();
            ui.add_space(SPACING);
            ui.label(egui::RichText::new("Overrides").size(14.0));
        }
    }
}

impl MonitorOverridePage {
    pub fn from_config(_config: &SsbConfig) -> Self {
        Self {}
    }

    fn render_monitors(&mut self, ui: &mut egui::Ui, results: &ApplyResults) {
        if results.monitors.is_empty() {
            ui.label(egui::RichText::new("No devices found").color(egui::Color32::RED));
            return;
        }
        let properties = enum_iterator::all::<MonitorProperty>().collect::<Vec<_>>();

        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::Grid::new("monitors_properties_grid")
                .striped(true)
                .num_columns(properties.len())
                .show(ui, |ui| {
                    // Header row
                    for p in &properties {
                        ui.label(p.as_str());
                    }
                    ui.end_row();
                    // Monitor rows
                    for monitor in &results.monitors {
                        let property_map = monitor.properties.to_map();
                        for property in &properties {
                            if let Some(value) = property_map.get(property) {
                                let value = value.to_string();
                                let button = egui::Button::new(value.as_str().truncate_ellipse(24))
                                    .frame(false);
                                let hover = |ui: &mut egui::Ui| {
                                    ui.label(format!("\"{}\"", value));
                                    ui.label("Click to copy 📋");
                                };
                                if ui.add(button).on_hover_ui(hover).clicked() {
                                    ui.output_mut(|o| o.copied_text = value);
                                }
                            } else {
                                ui.label("");
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    }
}
