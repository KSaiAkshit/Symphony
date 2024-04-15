use eframe::{egui, epaint::Vec2, run_native, NativeOptions};
use symphony::gui::Symphony;

fn main() -> anyhow::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(54., 96.)),
        ..Default::default()
    };
    run_native("MyApp", options, Box::new(|_cc| Box::<Symphony>::default()))
        .expect("Starting app from here");
    Ok(())
}
