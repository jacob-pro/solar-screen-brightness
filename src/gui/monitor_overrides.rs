use crate::apply::ApplyResults;
use crate::config::{BrightnessValues, MonitorOverride, MonitorProperty, SsbConfig};
use crate::controller::Message;
use crate::gui::app::{save_config, set_red_widget_border, AppState, Page, SPACING};
use crate::gui::status::no_devices_found;
use ellipse::Ellipse;

const MAX_OVERRIDES: usize = 10;

pub struct MonitorOverridePage {
    overrides: Vec<Override>,
}

struct Override {
    key: MonitorProperty,
    pattern: String,
    disable: bool,
    day: u32,
    night: u32,
}

impl Override {
    fn is_valid(&self) -> bool {
        !self.pattern.is_empty()
    }
}

impl Page for MonitorOverridePage {
    fn render(&mut self, ui: &mut egui::Ui, app_state: &mut AppState) {
        {
            let results = app_state.results.read().unwrap();
            if let Some(results) = results.as_ref() {
                self.render_monitors(ui, results);
                ui.add_space(SPACING);
                ui.separator();
            }
        }
        ui.add_space(SPACING);
        self.render_overrides(ui, app_state);
    }
}

impl MonitorOverridePage {
    pub fn from_config(config: &SsbConfig) -> Self {
        let overrides = config
            .overrides
            .iter()
            .map(|o| Override {
                key: o.key,
                pattern: o.pattern.clone(),
                disable: o.brightness.is_none(),
                day: o.brightness.map(|b| b.brightness_day).unwrap_or(100),
                night: o.brightness.map(|b| b.brightness_night).unwrap_or(60),
            })
            .collect();
        Self { overrides }
    }

    fn copy_to_config(&self, config: &mut SsbConfig) {
        config.overrides = self
            .overrides
            .iter()
            .map(|o| MonitorOverride {
                pattern: o.pattern.clone(),
                key: o.key,
                brightness: (!o.disable).then_some(BrightnessValues {
                    brightness_day: o.day,
                    brightness_night: o.night,
                }),
            })
            .collect();
    }

    fn is_valid(&self) -> bool {
        self.overrides.iter().all(|o| o.is_valid())
    }

    fn render_monitors(&mut self, ui: &mut egui::Ui, results: &ApplyResults) {
        ui.label(egui::RichText::new("Monitor Properties").size(14.0));
        ui.add_space(SPACING);

        if results.monitors.is_empty() {
            ui.add(no_devices_found());
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

    fn render_overrides(&mut self, ui: &mut egui::Ui, app_state: &mut AppState) {
        ui.label(egui::RichText::new("Overrides").size(14.0));
        ui.add_space(SPACING);
        ui.label("Create monitor overrides that match one of the above monitor properties.");
        ui.label("The first override that is matched will be applied to the monitor.");
        ui.add_space(SPACING);

        let properties = enum_iterator::all::<MonitorProperty>().collect::<Vec<_>>();

        if !self.overrides.is_empty() {
            egui::Grid::new("overrides_grid")
                .striped(true)
                .num_columns(8)
                .min_col_width(0.0)
                .show(ui, |ui| {
                    ui.label("");
                    ui.label("");
                    ui.label("Property");
                    ui.label("Pattern")
                        .on_hover_text("You can use * as a wildcard match");
                    ui.label("Disable")
                        .on_hover_text("Disable automatic brightness");
                    ui.label("Day");
                    ui.label("Night");
                    ui.label("");
                    ui.end_row();

                    let last_idx = self.overrides.len() - 1;
                    for idx in 0..self.overrides.len() {
                        if ui
                            .add_enabled(idx != 0, egui::Button::new("⬆"))
                            .on_hover_text("Move up")
                            .clicked()
                        {
                            self.overrides.swap(idx, idx - 1);
                        }
                        if ui
                            .add_enabled(idx != last_idx, egui::Button::new("⬇"))
                            .on_hover_text("Move down")
                            .clicked()
                        {
                            self.overrides.swap(idx, idx + 1);
                        }
                        let o = self.overrides.get_mut(idx).unwrap();
                        egui::ComboBox::from_id_source(format!("override_key {}", idx))
                            .selected_text(o.key.as_str())
                            .show_ui(ui, |ui| {
                                for property in &properties {
                                    ui.selectable_value(&mut o.key, *property, property.as_str());
                                }
                            });

                        ui.add_enabled_ui(true, |ui| {
                            if o.pattern.is_empty() {
                                set_red_widget_border(ui);
                            }
                            ui.add(
                                egui::TextEdit::singleline(&mut o.pattern)
                                    .min_size(egui::vec2(140.0, 0.0)),
                            );
                        });

                        ui.add(egui::Checkbox::without_text(&mut o.disable));

                        if o.disable {
                            ui.label("N/A");
                            ui.label("N/A");
                        } else {
                            ui.add(
                                egui::DragValue::new(&mut o.day)
                                    .clamp_range(0u32..=100u32)
                                    .suffix("%"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut o.night)
                                    .clamp_range(0u32..=100u32)
                                    .suffix("%"),
                            );
                        }

                        if ui.button("❌").on_hover_text("Remove override").clicked() {
                            self.overrides.remove(idx);
                            return; // Important to avoid invalid index on next iteration
                        };

                        ui.end_row();
                    }
                });
            ui.add_space(SPACING);
        }

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.overrides.len() < MAX_OVERRIDES,
                    egui::Button::new("Create new override"),
                )
                .clicked()
            {
                self.overrides.push(Override {
                    key: MonitorProperty::DeviceName,
                    pattern: "".to_string(),
                    disable: false,
                    day: 100,
                    night: 60,
                })
            }
            if ui
                .add_enabled(self.is_valid(), egui::Button::new("Save"))
                .clicked()
            {
                let mut config = app_state.config.write().unwrap();
                self.copy_to_config(&mut config);
                app_state
                    .controller
                    .send(Message::Refresh("Override change"))
                    .unwrap();
                save_config(&mut config, &app_state.transitions);
            }
        });

        ui.add_space(SPACING);
    }
}
