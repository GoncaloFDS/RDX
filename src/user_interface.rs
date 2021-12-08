use egui::paint::Shadow;
use egui::Visuals;
use egui_winit::winit::window::Window;
use glam::Vec3;
use std::rc::Rc;
use winit::event::WindowEvent;

#[derive(Debug, PartialEq)]
pub enum EguiTheme {
    Dark,
    Light,
}

pub struct Settings {
    pub theme: EguiTheme,
    pub rotation: f32,
    pub light_position: Vec3,
    pub text: String,
}

pub struct UserInterface {
    egui: egui::CtxRef,
    egui_state: egui_winit::State,
    window: Rc<Window>,
    output: egui::Output,
    clipped_meshes: Vec<egui::ClippedMesh>,
    settings: Settings,
    display_settings: bool,
    display_profiler: bool,
}

impl UserInterface {
    pub fn new(window: Rc<Window>) -> Self {
        let egui = egui::CtxRef::default();
        let egui_state = egui_winit::State::new(&window);
        let mut visual = Visuals::default();
        visual.window_shadow = Shadow {
            extrusion: 0.0,
            color: Default::default(),
        };
        egui.set_visuals(visual);
        UserInterface {
            egui,
            egui_state,
            window,
            output: Default::default(),
            clipped_meshes: vec![],
            settings: Settings {
                theme: EguiTheme::Dark,
                rotation: 0.0,
                light_position: Default::default(),
                text: "porreiro pah".to_string(),
            },
            display_settings: false,
            display_profiler: false,
        }
    }

    pub fn on_event(&mut self, window_event: &WindowEvent) -> bool {
        self.egui_state.on_event(&self.egui, window_event)
    }

    pub fn update(&mut self) {
        puffin::profile_function!();
        self.begin_frame();

        if self.display_profiler {
            puffin_egui::profiler_window(&self.egui);
        }
        if self.display_settings {
            self.draw_settings();
        }

        self.end_frame();
    }

    fn begin_frame(&mut self) {
        self.egui
            .begin_frame(self.egui_state.take_egui_input(&self.window))
    }

    fn end_frame(&mut self) {
        let (output, clipped_shapes) = self.egui.end_frame();
        let clipped_meshes = self.egui.tessellate(clipped_shapes);
        self.output = output;
        self.clipped_meshes = clipped_meshes;
    }

    fn draw_settings(&mut self) {
        egui::Window::new("My Window")
            .resizable(true)
            .show(&self.egui, |ui| {
                ui.heading("Hello");
                ui.label("Hello egui!");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Theme");
                    let id = ui.make_persistent_id("theme_combo_box_window");
                    egui::ComboBox::from_id_source(id)
                        .selected_text(format!("{:?}", self.settings.theme))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.settings.theme, EguiTheme::Dark, "Dark");
                            ui.selectable_value(
                                &mut self.settings.theme,
                                EguiTheme::Light,
                                "Light",
                            );
                        });
                });
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

impl UserInterface {
    pub fn egui(&self) -> &egui::CtxRef {
        &self.egui
    }

    pub fn output(&self) -> &egui::Output {
        &self.output
    }

    pub fn clipped_meshes(&self) -> &[egui::ClippedMesh] {
        &self.clipped_meshes
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn settings_as_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub fn toggle_settings(&mut self) {
        self.display_settings = !self.display_settings;
    }

    pub fn toggle_profiler(&mut self) {
        self.display_profiler = !self.display_profiler;
    }
}
