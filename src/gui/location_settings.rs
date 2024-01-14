use crate::config::{Location, SsbConfig};
use crate::controller::Message;
use crate::gui::app::{set_red_widget_border, AppState, MessageModal, Modal, Page, SPACING};
use crate::gui::UserEvent;
use egui::{Context, Widget};
use geocoding::{Forward, GeocodingError, Openstreetmap};
use validator::Validate;

pub struct LocationSettingsPage {
    latitude: String,
    longitude: String,
    search_query: String,
}

impl LocationSettingsPage {
    pub fn from_config(config: &SsbConfig) -> Self {
        let location = config.location.as_ref();
        Self {
            latitude: coord_to_string(location.map(|l| l.latitude).unwrap_or(0.0)),
            longitude: coord_to_string(location.map(|l| l.longitude).unwrap_or(0.0)),
            search_query: String::default(),
        }
    }

    fn copy_to_config(&self, config: &mut SsbConfig) {
        config.location = Some(Location {
            latitude: self.latitude.parse().unwrap(),
            longitude: self.longitude.parse().unwrap(),
        });
        assert!(config.validate().is_ok())
    }

    fn is_latitude_valid(&self) -> bool {
        self.latitude
            .parse::<f64>()
            .is_ok_and(|l| (-90.0..=90.0).contains(&l))
    }

    fn is_longitude_valid(&self) -> bool {
        self.longitude
            .parse::<f64>()
            .is_ok_and(|l| (-180.0..=180.0).contains(&l))
    }
}

impl Page for LocationSettingsPage {
    fn render(&mut self, ui: &mut egui::Ui, app_state: &mut AppState) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Enter a city or location name"),
            );
            if ui
                .add_enabled(!self.search_query.is_empty(), egui::Button::new("Search"))
                .on_hover_text("Search using OpenStreetMap")
                .clicked()
            {
                on_search(app_state, self.search_query.clone());
            }
        });

        ui.add_space(SPACING);

        let latitude_valid = self.is_latitude_valid();
        let longitude_valid = self.is_longitude_valid();

        egui::Grid::new("location_settings")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Latitude")
                    .on_hover_text("Latitude (N), between -90° to 90°");
                ui.vertical(|ui| {
                    if !latitude_valid {
                        set_red_widget_border(ui);
                    }
                    ui.text_edit_singleline(&mut self.latitude);
                });
                ui.end_row();

                ui.label("Longitude")
                    .on_hover_text("Longitude (E), between -180° to 180°");
                ui.vertical(|ui| {
                    if !longitude_valid {
                        set_red_widget_border(ui);
                    }
                    ui.text_edit_singleline(&mut self.longitude);
                });
                ui.end_row();
            });

        ui.add_space(SPACING);

        let save_enabled = latitude_valid && longitude_valid;
        if ui
            .add_enabled(save_enabled, egui::Button::new("Save"))
            .clicked()
        {
            let mut config = app_state.config.write().unwrap();
            self.copy_to_config(&mut config);
            app_state
                .controller
                .send(Message::Refresh("Location change"))
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
        }
    }
}

fn coord_to_string(coord: f64) -> String {
    format!("{:.5}", coord)
}

struct SpinnerModal;

impl Modal for SpinnerModal {
    fn render(&self, ctx: &Context, _: &mut AppState) {
        egui::Area::new("SpinnerModal")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Spinner::new().size(30.0).ui(ui);
            });
    }
}

fn on_search(app_state: &mut AppState, search_string: String) {
    // Show the spinner modal
    app_state
        .transitions
        .queue_state_transition(|app| app.modal = Some(Box::new(SpinnerModal)));

    // Start a background thread
    let transitions = app_state.transitions.clone();
    let proxy = app_state.main_loop.clone();
    std::thread::spawn(move || {
        let result = Openstreetmap::new()
            .forward(&search_string)
            .map_err(|x| match x {
                GeocodingError::Request(r) => anyhow::Error::from(r),
                _ => anyhow::Error::from(x),
            });
        match result.map(|e| e.into_iter().next()) {
            Ok(None) => transitions.queue_state_transition(move |app| {
                app.modal = Some(Box::new(MessageModal {
                    title: "No results".to_string(),
                    message: format!("No location could be found for '{}'", search_string),
                }))
            }),
            Ok(Some(p)) => {
                transitions.queue_state_transition(move |app| {
                    app.location_settings_page.latitude = coord_to_string(p.y());
                    app.location_settings_page.longitude = coord_to_string(p.x());
                    app.modal = None;
                });
            }
            Err(e) => {
                log::error!("Error searching for location: {:?}", e);
                transitions.queue_state_transition(move |app| {
                    app.modal = Some(Box::new(MessageModal {
                        title: "Error".to_string(),
                        message: format!("Error occurred searching for location: {}", e),
                    }))
                });
            }
        }

        // Refresh the UI
        proxy
            .send_event(UserEvent::RepaintNow("Location Search Completed"))
            .unwrap();
    });
}
