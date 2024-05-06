#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use eframe::egui;

#[derive(Debug, Default)]
struct ArenaParserGUI {
    picked_dir: Option<String>
}

impl eframe::App for ArenaParserGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Arena Parser GUI");
            ui.horizontal(|ui| {
                ui.label("Pick a directory:");
                if ui.button("Pick").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.picked_dir = Some(path.display().to_string());
                    }
                }
            });
            if let Some(dir) = &self.picked_dir {
                ui.label(format!("Picked directory: {}", dir));
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0]) // wide enough for the drag-drop overlay text
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Box::<ArenaParserGUI>::default()),
    ).expect("TODO: panic message");
}
