use egui_winit::winit::window::Window;
use glam::Vec3;
use std::rc::Rc;
use winit::event::WindowEvent;

#[derive(Debug, PartialEq)]
enum EguiTheme {
    Dark,
    Light,
}
pub struct Settings {
    theme: EguiTheme,
    rotation: f32,
    light_position: Vec3,
    text: String,
}
pub struct UserInterface {
    egui: egui::CtxRef,
    egui_state: egui_winit::State,
    window: Rc<Window>,
    output: egui::Output,
    clipped_meshes: Vec<egui::ClippedMesh>,
    settings: Settings,
}

impl UserInterface {
    pub fn output(&self) -> &egui::Output {
        &self.output
    }

    pub fn clipped_meshes(&self) -> &[egui::ClippedMesh] {
        &self.clipped_meshes
    }

    pub fn new(window: Rc<Window>) -> Self {
        let egui = egui::CtxRef::default();
        let egui_state = egui_winit::State::new(&window);
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
                text: "".to_string(),
            },
        }
    }

    pub fn on_event(&mut self, window_event: &WindowEvent) -> bool {
        self.egui_state.on_event(&self.egui, window_event)
    }

    pub fn render(&mut self) {
        self.begin_frame();
        self.draw_settings();
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
