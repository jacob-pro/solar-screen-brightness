pub mod app;
mod brightness_settings;
mod help;
mod location_settings;
mod monitor_overrides;
mod status;

use crate::common::APP_NAME;
use crate::gui::app::SsbEguiApp;
use crate::tray::read_icon;
use egui_wgpu::wgpu::PowerPreference;
use egui_winit::winit;
use egui_winit::winit::event::{Event, WindowEvent};
use egui_winit::winit::event_loop::{EventLoopProxy, EventLoopWindowTarget};
use egui_winit::winit::window::Icon;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug)]
pub enum NextPaint {
    /// Wait for an event
    Wait,
    /// Queues a repaint for once the event loop handles its next redraw. Exists
    /// so that multiple input events can be handled in one frame.
    RepaintNext,
    /// Repaint at a particular time
    RepaintAt(Instant),
    /// Exit the event loop
    Exit,
}

#[derive(Debug)]
pub enum UserEvent {
    // When the tray button is clicked - we should open or bring forward the window
    OpenWindow(&'static str),
    // When the tray exit button is clicked - the application should exit
    Exit(&'static str),
    // Hide the window if it is open
    CloseWindow(&'static str),
    // Repaint now
    RepaintNow(&'static str),

    RequestRepaint {
        when: Instant,
        /// What the frame number was when the repaint was _requested_.
        frame_nr: u64,
    },
}

struct WgpuWinitRunning {
    painter: egui_wgpu::winit::Painter,
    app: SsbEguiApp,
    window: winit::window::Window,
    follow_system_theme: bool,
    egui_ctx: egui::Context,
    pending_full_output: egui::FullOutput,
    egui_winit: egui_winit::State,
}

impl Drop for WgpuWinitRunning {
    fn drop(&mut self) {
        self.painter.destroy();
    }
}

pub struct WgpuWinitApp<F> {
    repaint_proxy: EventLoopProxy<UserEvent>,
    running: Option<WgpuWinitRunning>,
    icon: Icon,
    app_factory: F,
    start_minimised: bool,
}

impl<F: Fn() -> SsbEguiApp> WgpuWinitApp<F> {
    pub fn new(
        event_loop: EventLoopProxy<UserEvent>,
        start_minimised: bool,
        app_factory: F,
    ) -> Self {
        if start_minimised {
            log::info!("Starting minimised");
        }
        let (buf, info) = read_icon();
        let icon = Icon::from_rgba(buf, info.width, info.height).unwrap();
        Self {
            repaint_proxy: event_loop,
            running: None,
            icon,
            app_factory,
            start_minimised,
        }
    }

    pub fn launch_window(&mut self, event_loop: &EventLoopWindowTarget<UserEvent>) -> NextPaint {
        let window = winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title(APP_NAME)
            .with_inner_size(winit::dpi::PhysicalSize {
                width: 1024,
                height: 768,
            })
            .with_window_icon(Some(self.icon.clone()))
            .build(event_loop)
            .unwrap();

        window.set_ime_allowed(true);

        let wgpu_config = egui_wgpu::WgpuConfiguration {
            power_preference: PowerPreference::LowPower,
            ..Default::default()
        };
        let mut painter = egui_wgpu::winit::Painter::new(wgpu_config, 1, None, false);

        pollster::block_on(painter.set_window(Some(&window))).unwrap();

        let mut egui_winit = egui_winit::State::new(event_loop);

        let max_texture_side = painter.max_texture_side().unwrap_or(2048);
        egui_winit.set_max_texture_side(max_texture_side);

        let native_pixels_per_point = window.scale_factor() as f32;
        egui_winit.set_pixels_per_point(native_pixels_per_point * 1.5);

        let egui_ctx = egui::Context::default();

        let system_theme = window.theme();
        egui_ctx.set_visuals(
            system_theme
                .map(visuals_from_winit_theme)
                .unwrap_or(egui::Visuals::light()),
        );

        let event_loop_proxy = Arc::new(Mutex::new(self.repaint_proxy.clone()));
        egui_ctx.set_request_repaint_callback(move |info| {
            let when = Instant::now() + info.after;
            let frame_nr = info.current_frame_nr;
            event_loop_proxy
                .lock()
                .unwrap()
                .send_event(UserEvent::RequestRepaint { when, frame_nr })
                .ok();
        });

        self.running = Some(WgpuWinitRunning {
            painter,
            app: (self.app_factory)(),
            window,
            follow_system_theme: system_theme.is_some(),
            egui_ctx,
            pending_full_output: Default::default(),
            egui_winit,
        });

        NextPaint::RepaintNext
    }

    pub fn frame_nr(&self) -> u64 {
        self.running.as_ref().map_or(0, |r| r.egui_ctx.frame_nr())
    }

    pub fn window(&self) -> Option<&winit::window::Window> {
        self.running.as_ref().map(|r| &r.window)
    }

    pub fn close_window(&mut self) -> NextPaint {
        // When the window is closed, we destroy the Window, but leave app running
        self.running = None;
        NextPaint::Wait
    }

    pub fn paint(&mut self) -> NextPaint {
        if let Some(running) = &mut self.running {
            let raw_input = running.egui_winit.take_egui_input(&running.window);

            // Run user code:
            let full_output = running.egui_ctx.run(raw_input, |egui_ctx| {
                running.app.update(egui_ctx);
            });

            running.pending_full_output.append(full_output);
            let full_output = std::mem::take(&mut running.pending_full_output);

            let egui::FullOutput {
                platform_output,
                repaint_after,
                textures_delta,
                shapes,
            } = full_output;

            running.egui_winit.handle_platform_output(
                &running.window,
                &running.egui_ctx,
                platform_output,
            );

            let clipped_primitives = { running.egui_ctx.tessellate(shapes) };

            let clear_color =
                egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32();

            running.painter.paint_and_update_textures(
                running.egui_ctx.pixels_per_point(),
                clear_color,
                &clipped_primitives,
                &textures_delta,
                false,
            );

            if repaint_after.is_zero() {
                NextPaint::RepaintNext
            } else if let Some(repaint_after_instant) = Instant::now().checked_add(repaint_after) {
                NextPaint::RepaintAt(repaint_after_instant)
            } else {
                NextPaint::Wait
            }
        } else {
            NextPaint::Wait
        }
    }

    pub fn on_event(
        &mut self,
        event_loop: &EventLoopWindowTarget<UserEvent>,
        event: &Event<'_, UserEvent>,
    ) -> NextPaint {
        match event {
            Event::Resumed if self.running.is_none() && !self.start_minimised => {
                self.launch_window(event_loop)
            }

            Event::UserEvent(UserEvent::Exit(src)) => {
                log::info!("Received Exit action from '{src}'");
                NextPaint::Exit
            }

            Event::UserEvent(UserEvent::CloseWindow(src)) => {
                log::info!("Received CloseWindow action from '{src}'");
                self.close_window()
            }

            Event::UserEvent(UserEvent::RepaintNow(src)) => {
                log::info!("Received RepaintNow action from '{src}'");
                NextPaint::RepaintNext
            }

            Event::UserEvent(UserEvent::OpenWindow(src)) => {
                log::info!("Received OpenWindow action from '{src}'");
                if let Some(window) = self.window() {
                    window.set_minimized(false);
                    window.focus_window();
                    NextPaint::Wait
                } else {
                    self.launch_window(event_loop)
                }
            }

            Event::UserEvent(UserEvent::RequestRepaint { when, frame_nr }) => {
                if self.frame_nr() == *frame_nr {
                    NextPaint::RepaintAt(*when)
                } else {
                    // old request - we've already repainted
                    NextPaint::Wait
                }
            }

            Event::WindowEvent { event, .. } => {
                if let Some(running) = &mut self.running {
                    match &event {
                        WindowEvent::Resized(physical_size) => {
                            // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                            // See: https://github.com/rust-windowing/winit/issues/208
                            // This solves an issue where the app would panic when minimizing on Windows.
                            if physical_size.width > 0 && physical_size.height > 0 {
                                running
                                    .painter
                                    .on_window_resized(physical_size.width, physical_size.height);
                            }
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            running
                                .painter
                                .on_window_resized(new_inner_size.width, new_inner_size.height);
                        }
                        WindowEvent::CloseRequested => {
                            return self.close_window();
                        }
                        WindowEvent::ThemeChanged(winit_theme) if running.follow_system_theme => {
                            let visuals = visuals_from_winit_theme(*winit_theme);
                            running.egui_ctx.set_visuals(visuals);
                        }
                        _ => {}
                    };

                    let event_response = running.egui_winit.on_event(&running.egui_ctx, event);

                    if event_response.repaint {
                        NextPaint::RepaintNext
                    } else {
                        NextPaint::Wait
                    }
                } else {
                    NextPaint::Wait
                }
            }

            _ => NextPaint::Wait,
        }
    }
}

fn visuals_from_winit_theme(theme: winit::window::Theme) -> egui::Visuals {
    match theme {
        winit::window::Theme::Dark => egui::Visuals::dark(),
        winit::window::Theme::Light => egui::Visuals::light(),
    }
}
