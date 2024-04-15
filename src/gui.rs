use eframe::egui::{self, Label, RichText, ScrollArea, TextStyle, TopBottomPanel};
use egui_plot::{Line, Plot, PlotPoints};

#[derive(Default, Debug)]
struct MenuOptions {
    show_serial_monitor: bool,
}

#[derive(Default, Debug)]
pub struct Symphony {
    n_items: usize,
    menu_options: MenuOptions,
    data: Vec<[f64; 2]>, // NOTE: This is f64 because egui_plot::PlotPoints needs it to be f64
    raw_data: Box<[u8]>,
}

impl Symphony {
    pub fn new() -> Self {
        Self {
            n_items: 0,
            menu_options: MenuOptions::default(),
            data: Vec::default(),
            raw_data: Box::default(),
        }
    }
    fn show_serial_monitor(&self, ctx: &egui::Context, is_expanded: bool) {
        TopBottomPanel::bottom("Serial Monitor").show_animated(ctx, is_expanded, |ui| {
            ui.add_space(10.);
            ui.add(Label::new(RichText::new("Serial Monitor").underline()));
            ui.add_space(10.);
            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);

            ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .stick_to_bottom(true)
                .show_rows(ui, row_height, self.n_items, |ui, row_range| {
                    for row in row_range {
                        let text = format!("This is row {}", row + 1);
                        ui.label(text);
                    }
                });
            ui.ctx().request_repaint();
        });
    }
}

impl eframe::App for Symphony {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("Menu Bar").show(ctx, |ui| {
            self.show_serial_monitor(ctx, true);
            // ui.group(|ui| {
            //     ui.checkbox(
            //         &mut self.menu_options.show_serial_monitor,
            //         "Enable serial monitor",
            //     );
            //     ui.set_visible(self.menu_options.show_serial_monitor);
            //     ui.add_enabled_ui(self.menu_options.show_serial_monitor, |ui| {
            //         if ui.button("Button that is not always clickable").clicked() {
            //             self.show_serial_monitor(ctx, true);
            //         }
            //     });
            // });
        });
        // TopBottomPanel::top("Serial Plotter").show_animated(ctx, true, |ui| {
        //     let data = PlotPoints::from(self.data.clone());
        //     let line = Line::new(data);
        //     Plot::new("my_plot")
        //         .view_aspect(2.0)
        //         .show(ui, |plot_ui| plot_ui.line(line));
        // });
        self.n_items += 1;
    }
}
