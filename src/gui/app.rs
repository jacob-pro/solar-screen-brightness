use crate::apply::ApplyResults;
use crate::config::SsbConfig;
use crate::controller::Message;
use crate::gui::brightness_settings::BrightnessSettingsPage;
use crate::gui::help::HelpPage;
use crate::gui::location_settings::LocationSettingsPage;
use crate::gui::status::StatusPage;
use crate::gui::UserEvent;
use egui::{Align, Color32, Layout, ScrollArea};
use egui_winit::winit::event_loop::EventLoopProxy;
use enum_iterator::Sequence;
use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};

pub const SPACING: f32 = 10.0;

pub trait Page {
    fn render(&mut self, ui: &mut egui::Ui, context: &mut AppState);
}

pub struct SsbEguiApp {
    selected_page: PageId,
    brightness_settings_page: BrightnessSettingsPage,
    pub location_settings_page: LocationSettingsPage,
    help_page: HelpPage,
    context: AppState,
    pub modal: Option<Box<dyn Modal>>,
}

pub struct AppState {
    pub main_loop: EventLoopProxy<UserEvent>,
    pub config: Arc<RwLock<SsbConfig>>,
    pub controller: Sender<Message>,
    pub results: Arc<RwLock<Option<ApplyResults>>>,
    pub transitions: Transitions,
}

pub type TransitionFn = Box<dyn FnOnce(&mut SsbEguiApp) + Send>;

#[derive(Default, Clone)]
pub struct Transitions(Arc<Mutex<VecDeque<TransitionFn>>>);

impl Transitions {
    pub fn queue_state_transition<F: FnOnce(&mut SsbEguiApp) + Send + 'static>(
        &self,
        transition: F,
    ) {
        self.0.lock().unwrap().push_back(Box::new(transition));
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Sequence)]
enum PageId {
    Status,
    BrightnessSettings,
    LocationSettings,
    Help,
}

impl PageId {
    fn title(self) -> &'static str {
        match self {
            PageId::Status => "Status",
            PageId::BrightnessSettings => "Brightness Settings",
            PageId::LocationSettings => "Location Settings",
            PageId::Help => "Help",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            PageId::Status => "ℹ",
            PageId::BrightnessSettings => "🔅",
            PageId::LocationSettings => "🌐",
            PageId::Help => "❔",
        }
    }
}

pub trait Modal {
    fn render(&self, ctx: &egui::Context, app_state: &mut AppState);
}

pub struct ExitModal;

impl Modal for ExitModal {
    fn render(&self, ctx: &egui::Context, app_state: &mut AppState) {
        egui::Window::new("Are you sure?")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(
                    "Exiting the application will stop the brightness being automatically updated",
                );
                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    if ui.button("Exit").clicked() {
                        app_state
                            .main_loop
                            .send_event(UserEvent::Exit("Exit Modal"))
                            .unwrap();
                    }
                    if ui.button("Cancel").clicked() {
                        app_state.transitions.queue_state_transition(|app| {
                            app.modal = None;
                        });
                    };
                });
            });
    }
}

pub struct MessageModal {
    pub title: String,
    pub message: String,
}

impl Modal for MessageModal {
    fn render(&self, ctx: &egui::Context, app_state: &mut AppState) {
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(&self.message);
                if ui.button("Ok").clicked() {
                    app_state.transitions.queue_state_transition(|app| {
                        app.modal = None;
                    });
                };
            });
    }
}

impl SsbEguiApp {
    pub fn new(
        main_loop: EventLoopProxy<UserEvent>,
        results: Arc<RwLock<Option<ApplyResults>>>,
        config: Arc<RwLock<SsbConfig>>,
        controller: Sender<Message>,
    ) -> Self {
        let config_read = config.read().unwrap();
        SsbEguiApp {
            selected_page: PageId::Status,
            brightness_settings_page: BrightnessSettingsPage::from_config(&config_read),
            location_settings_page: LocationSettingsPage::from_config(&config_read),
            modal: None,
            context: AppState {
                main_loop,
                config: config.clone(),
                controller,
                results,
                transitions: Default::default(),
            },
            help_page: Default::default(),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        let queue = self.context.transitions.0.clone();
        while let Some(transition) = queue.lock().unwrap().pop_front() {
            (transition)(self);
        }

        if let Some(modal) = &self.modal {
            modal.render(ctx, &mut self.context);
        }

        egui::SidePanel::left("MenuPanel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_enabled(self.modal.is_none());
                self.render_menu_panel(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(self.modal.is_none());
            self.render_main(ui);
        });
    }

    fn render_menu_panel(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                enum_iterator::all::<PageId>().for_each(|page| {
                    let text = format!("{} {}", page.icon(), page.title());
                    ui.selectable_value(&mut self.selected_page, page, text);
                });

                ui.separator();

                if ui
                    .add(egui::Button::new("❌ Close Window").fill(Color32::TRANSPARENT))
                    .clicked()
                {
                    self.context
                        .main_loop
                        .send_event(UserEvent::CloseWindow("Menu Button"))
                        .unwrap();
                }

                if ui
                    .add(egui::Button::new("⏹ Exit Application").fill(Color32::TRANSPARENT))
                    .clicked()
                {
                    self.modal = Some(Box::new(ExitModal));
                }
            });
        });
    }

    fn render_main(&mut self, ui: &mut egui::Ui) {
        ScrollArea::both().show(ui, |ui| {
            ui.heading(self.selected_page.title());
            ui.add_space(SPACING);
            match self.selected_page {
                PageId::Status => StatusPage.render(ui, &mut self.context),
                PageId::BrightnessSettings => {
                    self.brightness_settings_page.render(ui, &mut self.context)
                }
                PageId::LocationSettings => {
                    self.location_settings_page.render(ui, &mut self.context)
                }
                PageId::Help => self.help_page.render(ui, &mut self.context),
            }
        });
    }
}
