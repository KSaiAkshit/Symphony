use std::{collections::VecDeque, fmt::Display, time::Instant};

use eframe::egui::{self, Align, CentralPanel, Color32, ScrollArea, TextStyle, TopBottomPanel};
use egui_plot::{PlotPoint, PlotPoints};
use serialib::Device;
use serialport::{FlowControl, Parity};
use tracing::{info, instrument, span, trace, warn};

const BAUD_RATES: [u32; 20] = [
    50, 75, 110, 134, 150, 200, 300, 600, 1200, 1800, 2400, 4800, 9600, 19200, 38400, 57600,
    115200, 230400, 460800, 500000,
];

#[derive(Debug, Default)]
struct Measurement {
    values: VecDeque<PlotPoint>,
    look_behind: usize,
}

impl Measurement {
    fn new_with_look_behind(look_behind: usize) -> Self {
        Self {
            values: VecDeque::new(),
            look_behind,
        }
    }

    fn add(&mut self, measurement: PlotPoint) {
        if let Some(last) = self.values.back() {
            if measurement.x < last.x {
                self.values.clear()
            }
        }

        self.values.push_back(measurement);
        let limit = self.values.back().unwrap().x - (self.look_behind as f64);
        while let Some(front) = self.values.front() {
            if front.x >= limit {
                break;
            }
            self.values.pop_front();
        }
    }

    fn plot_values(&self) -> PlotPoints {
        PlotPoints::Owned(Vec::from_iter(self.values.iter().copied()))
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
enum Delimiter {
    #[default]
    Space,
    Comma,
    Tab,
    Other(String),
}

impl Display for Delimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Delimiter::Space => write!(f, "Space"),
            Delimiter::Comma => write!(f, "Comma"),
            Delimiter::Tab => write!(f, "Tab"),
            Delimiter::Other(_) => write!(f, "Other"),
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
enum Panel {
    #[default]
    /// Configure the Port
    Port,
    /// Settings for the Graph
    Plot,
    /// Send Commands to the Device
    Commands,
    /// Settings for Recording the Plot
    Record,
    /// Text view of Serial Monitor
    TextView,
    /// Show event logging of the App
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
            Panel::Port => {
                write!(f, "Port")
            }
        }
    }
}

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

#[derive(Default, Debug, Clone)]
struct Command {
    cmd: String,
    fmt: bool,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = if self.fmt {
            String::from("HEX")
        } else {
            String::from("ASCII")
        };
        write!(f, "Command: '{}', Format: {}", self.cmd, format)
    }
}

#[derive(Default, Debug, PartialEq)]
struct PlotOptions {
    delimiter: Delimiter,
    buffer_size: usize,
    plot_width: usize,
    x_axis: [usize; 2],
    y_axis: [usize; 2],
    autoscale: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Symphony {
    n_items: usize,
    text_view_options: TextViewOptions,
    plot_options: PlotOptions,
    current_port: Device,
    connected: bool,
    // NOTE: Maybe use a VecDeque?
    plot_data: Measurement,
    raw_data: Vec<u8>,
    open_panel: Panel,
    commands: Vec<Command>,
    log: Vec<String>,
    absolute_time: Instant,
}

impl Symphony {
    #[instrument]
    pub fn new() -> Self {
        Self {
            n_items: 0,
            text_view_options: TextViewOptions::default(),
            plot_options: PlotOptions::default(),
            current_port: Device::default(),
            connected: false,
            plot_data: Measurement::new_with_look_behind(5),
            raw_data: Vec::default(),
            open_panel: Panel::default(),
            commands: vec![Command::default(), Command::default()],
            log: Vec::default(),
            absolute_time: Instant::now(),
        }
    }

    fn draw_plot(&mut self, ui: &mut egui::Ui) {
        let plot = egui_plot::Plot::new("measurements");
        // for y in self.include_y.iter() {
        //     plot = plot.include_y(*y);
        // }
        self.plot_data.add(
            [
                self.absolute_time.elapsed().as_millis() as f64 * 0.001,
                (self.absolute_time.elapsed().as_millis() as f64 * 0.001).sin(),
            ]
            .into(),
        );

        plot.show(ui, |plot_ui| {
            plot_ui.line(egui_plot::Line::new(self.plot_data.plot_values()));
        });
    }

    fn draw_bottom_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.open_panel, Panel::Port, "Port");
            ui.selectable_value(&mut self.open_panel, Panel::Log, "Log");
            ui.selectable_value(&mut self.open_panel, Panel::Plot, "Plot");
            ui.selectable_value(&mut self.open_panel, Panel::Commands, "Commands");
            ui.selectable_value(&mut self.open_panel, Panel::Record, "Record");
            ui.selectable_value(&mut self.open_panel, Panel::TextView, "TextView");
        });
        ui.separator();
        trace!("{}", self.open_panel);
        match self.open_panel {
            Panel::Port => {
                self.show_port_settings(ui);
            }
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

    fn show_port_settings(&mut self, ui: &mut egui::Ui) {
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
            info!("{:?}", &self.plot_options);
            // TODO: Connect to port here
            let port = self.current_port.open();
            match port {
                Ok(port) => {
                    self.connected = !self.connected;
                    // Have some kind of function to get connection status. A Lock
                    info!("Connected to port: {:?}", port);
                    self.log.push(format!("Connected to port: {:?}", port));
                }
                Err(e) => {
                    warn!(
                        "Error connecting to port: {:?}, because: {}",
                        &self.current_port, e
                    );
                    self.log.push(format!(
                        "Error connecting to port: {:?}, because: {}",
                        &self.current_port, e
                    ));
                    warn!("{}", &e);
                }
            }
        };
    }

    fn show_text_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Serial Monitor");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.text_view_options.auto_scroll, "AutoScroll");
                ui.checkbox(&mut self.text_view_options.time_stamp, "Time Stamps");
            })
        });
        ui.add_space(10.);
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);

        ScrollArea::vertical()
            .max_width(f32::INFINITY)
            .stick_to_bottom(self.text_view_options.auto_scroll)
            .auto_shrink(false)
            .show_rows(ui, row_height, self.n_items, |ui, row_range| {
                for row in row_range {
                    let text = match self.text_view_options.time_stamp {
                        true => {
                            format!(
                                "[{}.{}] This is row {}",
                                self.absolute_time.elapsed().as_secs(),
                                self.absolute_time.elapsed().subsec_millis(),
                                row + 1
                            )
                        }
                        false => format!("This is row {}", row + 1),
                    };
                    if self.text_view_options.auto_scroll {
                        ui.scroll_to_cursor(Some(Align::TOP));
                    }
                    ui.label(text);
                }
            });
    }

    fn show_plot_settings(&mut self, ui: &mut egui::Ui) {
        let x_range = self.plot_options.x_axis;
        let y_range = self.plot_options.y_axis;
        ui.horizontal_wrapped(|ui| {
            // INFO: Add some way to enforce minimum buffer size and min x axis range
            ui.label("Set Buffer Size");
            ui.add(egui::DragValue::new(&mut self.plot_options.buffer_size).range(0..=100_000));
            ui.add_space(15.);
            ui.label("Range for X-axis");
            // Min
            ui.add(egui::DragValue::new(&mut self.plot_options.x_axis[0]).range(0..=x_range[1]));
            // Max
            ui.add(
                egui::DragValue::new(&mut self.plot_options.x_axis[1])
                    .range(0..=self.plot_options.plot_width),
            );
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Set Plot Width ");
            ui.add(
                egui::DragValue::new(&mut self.plot_options.plot_width)
                    .range(0..=self.plot_options.buffer_size),
            );
            ui.add_space(15.);
            ui.label("Range for Y-axis");
            // Min
            ui.add(egui::DragValue::new(&mut self.plot_options.y_axis[0]).range(0..=y_range[1]));
            // Max
            ui.add(
                egui::DragValue::new(&mut self.plot_options.y_axis[1])
                    .range(0..=self.plot_options.plot_width),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Select Delimiter");
            egui::ComboBox::from_label("Delimiter")
                .selected_text(format!("{}", self.plot_options.delimiter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.plot_options.delimiter,
                        Delimiter::Space,
                        "Space",
                    );
                    ui.selectable_value(&mut self.plot_options.delimiter, Delimiter::Tab, "Tab");
                    ui.selectable_value(
                        &mut self.plot_options.delimiter,
                        Delimiter::Comma,
                        "Comma",
                    );
                    ui.selectable_value(
                        &mut self.plot_options.delimiter,
                        Delimiter::Other(String::default()),
                        "Other",
                    );
                });
            if let Delimiter::Other(ref mut custom) = self.plot_options.delimiter {
                ui.label("Custom Delimiter: ");
                ui.text_edit_singleline(custom);
            }
        });
    }

    fn show_commands(&mut self, ui: &mut egui::Ui) {
        if ui.button("Add Command").clicked() {
            self.commands.push(Command::default());
        }
        self.commands.iter_mut().enumerate().for_each(|(idx, c)| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Command {}", idx));
                ui.text_edit_singleline(&mut c.cmd);
                ui.toggle_value(&mut c.fmt, "ASCII/HEX");
                if ui.button("Send").clicked() {
                    // TODO Send command
                    info!("Sending Command {}", c);
                    self.log.push(format!("Sending Command {}", c));
                    c.cmd.clear()
                }
            });
        });
    }

    fn show_record_settings(&self, ui: &mut egui::Ui) {
        ui.label("Showing recording settings");
    }

    fn show_log(&self, ui: &mut egui::Ui) {
        self.log.iter().for_each(|line| {
            ui.label(line);
        })
    }
}

impl eframe::App for Symphony {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let span = span!(tracing::Level::INFO, "Update");
        let _guard = span.enter();
        TopBottomPanel::top("Plotting area")
            .resizable(true)
            .min_height(0.4 * ctx.available_rect().height())
            .max_height(0.95 * ctx.available_rect().height())
            .default_height(0.75 * ctx.available_rect().height())
            .show(ctx, |ui| {
                self.draw_plot(ui);
            });
        CentralPanel::default().show(ctx, |ui| {
            self.draw_bottom_panel(ui);
        });
        self.n_items += 1;
    }
}
