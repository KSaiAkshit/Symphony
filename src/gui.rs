use std::fmt::Display;

use eframe::egui::{
    self, Align, CentralPanel, Color32, ScrollArea, SidePanel, TextStyle, TopBottomPanel,
};
use egui_plot::{Line, Plot, PlotPoints};
use serialib::Device;
use serialport::{FlowControl, Parity};
use tracing::{info, trace};

const BAUD_RATES: [u32; 20] = [
    50, 75, 110, 134, 150, 200, 300, 600, 1200, 1800, 2400, 4800, 9600, 19200, 38400, 57600,
    115200, 230400, 460800, 500000,
];

#[derive(PartialEq, Eq, Debug)]
enum Panel {
    Plot,
    Commands,
    Record,
    TextView,
    Log,
}

impl Display for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Panel::Plot => {
                write!(f, "Plot")
            }
            Panel::Log => {
                write!(f, "Log")
            }
            Panel::Commands => {
                write!(f, "Commands")
            }
            Panel::Record => {
                write!(f, "Record")
            }
            Panel::TextView => {
                write!(f, "TextView")
            }
        }
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::Log
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct TextViewOptions {
    auto_scroll: bool,
    time_stamp: bool,
}

impl Default for TextViewOptions {
    fn default() -> Self {
        Self {
            auto_scroll: true,
            time_stamp: false,
        }
    }
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone)]
struct Command {
    cmd: String,
    fmt: bool,
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Symphony {
    n_items: usize,
    text_view_options: TextViewOptions,
    current_port: Device,
    connected: bool,
    // for plotting, this isn;t a great structure
    data: Vec<[f64; 2]>, // NOTE: This is f64 because egui_plot::PlotPoints needs it to be f64
    raw_data: Vec<u8>,
    open_panel: Panel,
    commands: Vec<Command>,
}

#[allow(dead_code)]
impl Symphony {
    pub fn new() -> Self {
        Self {
            n_items: 0,
            text_view_options: TextViewOptions::default(),
            current_port: Device::default(),
            connected: false,
            data: Vec::default(),
            raw_data: Vec::default(),
            open_panel: Panel::default(),
            commands: vec![Command::default(), Command::default()],
        }
    }

    fn draw_plot(&mut self, ui: &mut egui::Ui) {
        let sin: PlotPoints = (0..1000)
            .map(|i| {
                let x = i as f64 * 0.01;
                [x, x.sin()]
            })
            .collect();
        let line = Line::new(sin);
        Plot::new("my_plot")
            .view_aspect(2.0)
            .show(ui, |plot_ui| plot_ui.line(line));
    }

    fn draw_bottom_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.open_panel, Panel::Log, "Log");
            ui.selectable_value(&mut self.open_panel, Panel::Plot, "Plot");
            ui.selectable_value(&mut self.open_panel, Panel::Commands, "Commands");
            ui.selectable_value(&mut self.open_panel, Panel::Record, "Record");
            ui.selectable_value(&mut self.open_panel, Panel::TextView, "TextView");
        });
        ui.separator();
        trace!("{}", self.open_panel);
        match self.open_panel {
            Panel::Plot => {
                self.show_plot_settings(ui);
            }
            Panel::Commands => {
                self.show_commands(ui);
            }
            Panel::Record => {
                self.show_record_settings(ui);
            }
            Panel::TextView => {
                self.show_text_view(ui);
            }
            Panel::Log => {
                self.show_log(ui);
            }
        }
    }

    fn draw_side_panel(&mut self, ui: &mut egui::Ui) {
        let ports = serialib::get_serial_devices().unwrap();
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_label("Select port")
                .selected_text(self.current_port.path.clone())
                .show_ui(ui, |ui| {
                    ports.iter().for_each(|p| {
                        ui.selectable_value(&mut self.current_port.path, p.clone(), p);
                    });
                });
            egui::ComboBox::from_label("Set Baud Rate")
                .selected_text(format!("{}", self.current_port.baud_rate))
                .show_ui(ui, |ui| {
                    BAUD_RATES.iter().for_each(|b| {
                        ui.selectable_value(
                            &mut self.current_port.baud_rate,
                            *b as usize,
                            format!("{} ", b),
                        );
                    })
                });
        });
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_label("Choose parity")
                .selected_text(format!("{}", self.current_port.parity))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.current_port.parity, Parity::None, "None");
                    ui.selectable_value(&mut self.current_port.parity, Parity::Odd, "Odd");
                    ui.selectable_value(&mut self.current_port.parity, Parity::Even, "Even");
                });
            egui::ComboBox::from_label("Flow Control")
                .selected_text(format!("{}", self.current_port.flow_control))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.current_port.flow_control,
                        FlowControl::None,
                        "None",
                    );
                    ui.selectable_value(
                        &mut self.current_port.flow_control,
                        FlowControl::Software,
                        "Software",
                    );
                    ui.selectable_value(
                        &mut self.current_port.flow_control,
                        FlowControl::Hardware,
                        "Hardware",
                    );
                });
            egui::ComboBox::from_label("Stop Bits")
                .selected_text(format!("{}", self.current_port.stop_bits))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.current_port.stop_bits,
                        serialport::StopBits::One,
                        "One",
                    );
                    ui.selectable_value(
                        &mut self.current_port.stop_bits,
                        serialport::StopBits::Two,
                        "Two",
                    );
                });
        });
        let (response, col) = if self.connected {
            (String::from("Disconnect"), Color32::DARK_RED)
        } else {
            (String::from("Connect"), Color32::DARK_GREEN)
        };
        let response = ui.add(egui::Button::new(response).fill(col));
        if response.clicked() {
            self.connected = !self.connected;
            dbg!(&self.current_port);
            // TODO: Connect to port here
        };
    }

    fn show_text_view(&mut self, ui: &mut egui::Ui) {
        // ui.add_space(10.);
        ui.horizontal(|ui| {
            ui.label("Serial Monitor");
            ui.add_space(0.85 * ui.available_width());
            ui.checkbox(&mut self.text_view_options.auto_scroll, "AutoScroll");
            ui.checkbox(&mut self.text_view_options.time_stamp, "Time Stamps");
        });
        ui.add_space(10.);
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);

        ScrollArea::vertical()
            .max_width(f32::INFINITY)
            .stick_to_bottom(self.text_view_options.auto_scroll)
            .auto_shrink(false)
            .show_rows(ui, row_height, self.n_items, |ui, row_range| {
                // let time = std::time::Instant::now();
                // let mut duration = std::time::Duration::default();
                // TODO Figure out relative time somehow
                for row in row_range {
                    let text = match self.text_view_options.time_stamp {
                        true => {
                            format!("{} This is row {}", "TIME", row + 1)
                        }
                        false => format!("This is row {}", row + 1),
                    };
                    if self.text_view_options.auto_scroll {
                        ui.scroll_to_cursor(Some(Align::TOP));
                    }
                    ui.label(text);
                    // duration = time.elapsed();
                }
            });
        ui.ctx().request_repaint();
    }

    fn show_plot_settings(&self, ui: &mut egui::Ui) {
        ui.label("Showing plot settings");
    }

    fn show_commands(&mut self, ui: &mut egui::Ui) {
        self.commands.iter_mut().enumerate().for_each(|(idx, c)| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Command {}", idx));
                ui.text_edit_singleline(&mut c.cmd);
                ui.toggle_value(&mut c.fmt, "ASCII/HEX");
                if ui.button("Send").clicked() {
                    // TODO Send command
                    info!("Sending Command {}", c.cmd);
                }
            });
        })
    }

    fn show_record_settings(&self, ui: &mut egui::Ui) {
        ui.label("Showing recording settings");
    }

    fn show_log(&self, ui: &mut egui::Ui) {
        ui.label("Showing log");
    }
}

impl eframe::App for Symphony {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        SidePanel::right("Side Panel").show(ctx, |ui| {
            // INFO Maybe push this to bottom panel?
            self.draw_side_panel(ui);
        });
        TopBottomPanel::top("Plotting area").show(ctx, |ui| {
            self.draw_plot(ui);
        });
        CentralPanel::default().show(ctx, |ui| {
            self.draw_bottom_panel(ui);
            //     let data = PlotPoints::from(self.data.clone());
            //     let line = Line::new(data);
            //     Plot::new("my_plot")
            //         .view_aspect(2.0)
            //         .show(ui, |plot_ui| plot_ui.line(line));
        });
        self.n_items += 1;
    }
}
