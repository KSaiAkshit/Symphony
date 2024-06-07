use eframe::{egui::ViewportBuilder, epaint::Vec2, run_native, NativeOptions};
use symphony::gui::Symphony;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting App");
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(54., 96.)),
        ..Default::default()
    };
    run_native(
        "MyApp",
        options,
        Box::new(|_cc| Box::<Symphony>::new(Symphony::new())),
    )
    .expect("Starting app from here");
    Ok(())
}
