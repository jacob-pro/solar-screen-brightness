use crate::apply::ApplyResults;
use crate::gui::app::{AppState, Page, SPACING};
use chrono::{Local, TimeZone};

pub struct StatusPage;

impl Page for StatusPage {
    fn render(&mut self, ui: &mut egui::Ui, context: &mut AppState) {
        let results = context.results.read().unwrap();
        if let Some(results) = results.as_ref() {
            display_apply_results(results, ui);
        } else {
            let config = context.config.read().unwrap();
            if config.location.is_none() {
                ui.label("A location must be configured");
            } else {
                ui.label("Brightness controller has not yet started");
            }
        }
    }
}

fn display_apply_results(results: &ApplyResults, ui: &mut egui::Ui) {
    let date_format = "%H:%M %P (%b %d)";
    let sunrise = Local
        .timestamp_opt(results.sun.rise, 0)
        .unwrap()
        .format(date_format)
        .to_string();
    let sunset = Local
        .timestamp_opt(results.sun.set, 0)
        .unwrap()
        .format(date_format)
        .to_string();

    egui::Grid::new("sun_times_grid")
        .num_columns(2)
        .show(ui, |ui| {
            if results.sun.visible {
                ui.label("Sunrise");
                ui.label(sunrise);
                ui.end_row();
                ui.label("Sunset");
                ui.label(sunset);
                ui.end_row();
            } else {
                ui.label("Sunset");
                ui.label(sunset);
                ui.end_row();
                ui.label("Sunrise");
                ui.label(sunrise);
                ui.end_row();
            }
        });

    ui.add_space(SPACING);
    ui.separator();
    ui.add_space(SPACING);

    if results.monitors.is_empty() {
        ui.add(no_devices_found());
        return;
    }

    egui::Grid::new("monitors_grid")
        .striped(true)
        .num_columns(5)
        .show(ui, |ui| {
            ui.label("Name");
            ui.label("Day")
                .on_hover_text("Configured day time brightness");
            ui.label("Night")
                .on_hover_text("Configured night time brightness");
            ui.label("Now")
                .on_hover_text("The computed brightness percentage for this monitor");
            ui.label("Status");
            ui.label("Next update")
                .on_hover_text("Time that the brightness will be changed");
            ui.end_row();

            results.monitors.iter().for_each(|monitor| {
                ui.label(&monitor.properties.device_name);
                if let Some(brightness) = &monitor.brightness {
                    ui.label(format!("{}%", brightness.brightness_day));
                    ui.label(format!("{}%", brightness.brightness_night));
                    ui.label(format!("{}%", brightness.brightness));

                    match &monitor.error {
                        None => ui
                            .label("Ok")
                            .on_hover_text("Brightness was applied successfully"),
                        Some(e) => ui
                            .label(egui::RichText::new("Error").color(egui::Color32::RED))
                            .on_hover_text(e),
                    };

                    match brightness.expiry_time {
                        None => ui.label("Never"),
                        Some(expiry_time) => {
                            let changes_at = Local.timestamp_opt(expiry_time, 0).unwrap();
                            ui.label(changes_at.format("%H:%M %P").to_string())
                                .on_hover_text(changes_at.format("%b %d").to_string())
                        }
                    };
                } else {
                    (0..3).for_each(|_| {
                        ui.label("N/A");
                    });
                    ui.label("Disabled")
                        .on_hover_text("Dynamic brightness is disabled due to a monitor override");
                    ui.label("Never");
                }
                ui.end_row();
            });
        });
}

pub fn no_devices_found() -> egui::Label {
    egui::Label::new(egui::RichText::new("No devices found").color(egui::Color32::RED))
}
