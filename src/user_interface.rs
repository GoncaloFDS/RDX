use egui::Visuals;
use egui_winit::winit::window::Window;
use glam::Vec3;
use std::rc::Rc;
use winit::event::WindowEvent;

pub struct Settings {
    pub rotation: f32,
    pub light_position: Vec3,
    pub text: String,
}

pub struct UserInterface {
    egui: egui::Context,
    egui_state: egui_winit::State,
    output: egui::PlatformOutput,
    clipped_meshes: Vec<egui::ClippedMesh>,
    settings: Settings,
}

impl UserInterface {
    pub fn new(window: &Window) -> Self {
        let egui = egui::Context::default();
        let egui_state = egui_winit::State::new(2048, &window);
        let visual = Visuals::dark();
        egui.set_visuals(visual);
        UserInterface {
            egui,
            egui_state,
            output: Default::default(),
            clipped_meshes: vec![],
            settings: Settings {
                rotation: 0.0,
                light_position: Default::default(),
                text: "".to_string(),
            },
        }
    }

    pub fn on_event(&mut self, window_event: &WindowEvent) -> bool {
        self.egui_state.on_event(&self.egui, window_event)
    }

    pub fn update(&mut self) {}

    fn begin_frame(&mut self, window: &Window) {
        self.egui
            .begin_frame(self.egui_state.take_egui_input(window))
    }

    fn end_frame(&mut self) {
        let output = self.egui.end_frame();
        self.output = output.platform_output;
        self.clipped_meshes = self.egui.tessellate(output.shapes);
    }

    fn draw_settings(&mut self) {
        egui::Window::new("My Window")
            .resizable(true)
            .show(&self.egui, |ui| {
                ui.heading("Hello");
                ui.label("Hello egui!");
                ui.separator();
                ui.hyperlink("https://github.com/emilk/egui");
                ui.separator();
                ui.label("Rotation");
                ui.add(egui::widgets::DragValue::new(&mut self.settings.rotation));
                ui.add(egui::widgets::Slider::new(
                    &mut self.settings.rotation,
                    -180.0..=180.0,
                ));
                ui.label("Light Position");
                ui.horizontal(|ui| {
                    ui.label("x:");
                    ui.add(egui::widgets::DragValue::new(
                        &mut self.settings.light_position.x,
                    ));
                    ui.label("y:");
                    ui.add(egui::widgets::DragValue::new(
                        &mut self.settings.light_position.y,
                    ));
                    ui.label("z:");
                    ui.add(egui::widgets::DragValue::new(
                        &mut self.settings.light_position.z,
                    ));
                });
                ui.separator();
                ui.text_edit_singleline(&mut self.settings.text);
            });
    }
}
