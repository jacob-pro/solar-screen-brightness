use crate::calculator::calculate_brightness;
use crate::config::{Location, SsbConfig};
use crate::controller::Message;
use crate::gui::app::{save_config, AppState, Page, SPACING};
use chrono::{Duration, DurationRound, TimeZone};
use egui::plot::{uniform_grid_spacer, GridInput, GridMark, Line, PlotBounds};
use egui::widgets::plot::Plot;
use std::mem::take;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use sunrise_sunset_calculator::SunriseSunsetParameters;
use validator::Validate;

pub struct BrightnessSettingsPage {
    brightness_day: u32,
    brightness_night: u32,
    transition_mins: u32,
    plot: Option<PlotData>,
}

struct PlotData {
    points: Vec<[f64; 2]>,
    generated_at: SystemTime,
    brightness_day: u32,
    brightness_night: u32,
    transition_mins: u32,
    location: Location,
}

impl PlotData {
    fn is_stale(&self, config: &SsbConfig) -> bool {
        if let Some(location) = config.location {
            if location != self.location {
                return true;
            }
        }
        if config.brightness_day != self.brightness_day {
            return true;
        }
        if config.brightness_night != self.brightness_night {
            return true;
        }
        if config.transition_mins != self.transition_mins {
            return true;
        }
        let age = SystemTime::now().duration_since(self.generated_at).unwrap();
        if chrono::Duration::from_std(age).unwrap().num_minutes() > 5 {
            return true;
        }
        false
    }
}

impl BrightnessSettingsPage {
    pub fn from_config(config: &SsbConfig) -> Self {
        Self {
            brightness_day: config.brightness_day,
            brightness_night: config.brightness_night,
            transition_mins: config.transition_mins,
            plot: None,
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
                    .send(Message::Refresh("Brightness change"))
                    .unwrap();
            }
            if ui.button("Save").clicked() {
                let mut config = app_state.config.write().unwrap();
                self.copy_to_config(&mut config);
                app_state
                    .controller
                    .send(Message::Refresh("Brightness change"))
                    .unwrap();
                save_config(&mut config, &app_state.transitions);
            };
        });

        ui.add_space(SPACING);
        self.render_plot(ui, app_state);
    }
}

const LINE_NAME: &str = "Brightness";

impl BrightnessSettingsPage {
    fn render_plot(&mut self, ui: &mut egui::Ui, app_state: &mut AppState) {
        let config = app_state.config.read().unwrap();

        if let Some(location) = config.location {
            self.plot = Some(match take(&mut self.plot) {
                None => generate_plot_data(
                    location,
                    config.brightness_day,
                    config.brightness_night,
                    config.transition_mins,
                ),
                Some(x) if x.is_stale(&config) => generate_plot_data(
                    location,
                    config.brightness_day,
                    config.brightness_night,
                    config.transition_mins,
                ),
                Some(x) => x,
            });
        }

        if let Some(plot) = &self.plot {
            ui.separator();
            ui.add_space(SPACING);

            let first = plot.points.first().unwrap()[0];
            let last = plot.points.last().unwrap()[0];
            let line = Line::new(plot.points.clone())
                .name(LINE_NAME)
                .highlight(true);

            Plot::new("brightness_curve")
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .y_grid_spacer(uniform_grid_spacer(|_| [100.0, 20.0, 10.0]))
                .y_axis_formatter(|val, _| format!("{}%", val))
                .x_grid_spacer(x_grid_spacer)
                .label_formatter(|name, point| {
                    if name == LINE_NAME {
                        format!("{}\nBrightness {}%", convert_time(point.x), point.y)
                    } else {
                        String::new()
                    }
                })
                .x_axis_formatter(|val, _| convert_time(val))
                .show(ui, |plot_ui| {
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max([first, -5.0], [last, 105.0]));
                    plot_ui.line(line)
                });
        }
    }
}

fn convert_time(time: f64) -> String {
    let time = chrono::Local.timestamp_opt(time as i64, 0).unwrap();
    time.format("%I:%M %P").to_string()
}

const HOURS: i64 = 6;

// spaces the x-axis hourly
fn x_grid_spacer(input: GridInput) -> Vec<GridMark> {
    let min_unix = input.bounds.0 as i64;
    let max_unix = input.bounds.1 as i64;
    let min_local = chrono::Local.timestamp_opt(min_unix, 0).unwrap();
    let lowest_whole_hour = min_local.duration_trunc(Duration::hours(HOURS)).unwrap();

    let mut output = Vec::new();
    let hours_unix = HOURS * 3600;

    let mut rounded_unix = lowest_whole_hour.timestamp();
    while rounded_unix < max_unix {
        if rounded_unix >= min_unix {
            output.push(GridMark {
                value: rounded_unix as f64,
                step_size: hours_unix as f64,
            });
        }
        rounded_unix += hours_unix;
    }
    output
}

fn generate_plot_data(
    location: Location,
    brightness_day: u32,
    brightness_night: u32,
    transition_mins: u32,
) -> PlotData {
    log::debug!("Generating plot...");
    let timer_start = Instant::now();

    let now = SystemTime::now();
    let graph_start = (now - Duration::hours(2).to_std().unwrap())
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let graph_end = (now + Duration::hours(22).to_std().unwrap())
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut points = Vec::new();
    let mut current = graph_start;

    while current <= graph_end {
        let sun = SunriseSunsetParameters::new(current, location.latitude, location.longitude)
            .calculate()
            .unwrap();
        let brightness = calculate_brightness(
            brightness_day,
            brightness_night,
            transition_mins,
            &sun,
            current,
        );
        let next_time = brightness.expiry_time.unwrap_or(graph_end).min(graph_end);

        // Add some extra points in the "flat" zone to allow cursor to snap to the line
        // This is a bit of a hack, assuming if expiry is greater than 30 minutes,
        // to be completely accurate we would need to look ahead at the next calculation.
        if brightness.expiry_time.unwrap_or(i64::MAX) - current > 1800 {
            for second in num::range_step(current, next_time, 240) {
                points.push([second as f64, brightness.brightness as f64]);
            }
        } else {
            points.push([current as f64, brightness.brightness as f64]);
        }

        if current == graph_end {
            break;
        }
        current = next_time;
    }

    log::debug!(
        "Plot took {:?} {} points",
        timer_start.elapsed(),
        points.len()
    );

    PlotData {
        points,
        generated_at: now,
        brightness_day,
        brightness_night,
        transition_mins,
        location,
    }
}
