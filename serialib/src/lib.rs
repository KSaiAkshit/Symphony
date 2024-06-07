use serde::{Deserialize, Serialize};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::time::Duration;
use thiserror::Error;

mod test;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't find Serial Ports because {0}")]
    NoPortsAvailable(serialport::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialDevices {
    /// Hold all devices
    pub devices: Vec<Device>,
    /// To show in drop down
    pub labels: Vec<Vec<String>>,
    /// NOTE: No idea what this is for
    pub number_of_plots: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Name of device
    pub path: String,
    /// Communication speed in bits per second
    pub baud_rate: usize,
    /// Number of bits to represent one character of data
    pub data_bits: DataBits,
    /// Mode for managing data transmission rate
    pub flow_control: FlowControl,
    /// Parity to check whether data has been lost or written
    pub parity: Parity,
    /// Pattern of bits that indicate the end of a character or of the whole transmission
    pub stop_bits: StopBits,
    /// Allowed time to complete read and write operations
    pub timeout: Duration,
}

impl Default for SerialDevices {
    fn default() -> Self {
        Self {
            devices: Vec::default(),
            labels: Vec::default(),
            number_of_plots: vec![1],
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Self {
            path: String::default(),
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_millis(10),
        }
    }
}

impl Device {
    pub fn new(
        name: String,
        baud_rate: usize,
        data_bits: DataBits,
        flow_control: FlowControl,
        parity: Parity,
        stop_bits: StopBits,
        timeout: Duration,
    ) -> Self {
        Self {
            path: name,
            baud_rate,
            data_bits,
            flow_control,
            parity,
            stop_bits,
            timeout,
        }
    }
    /// Set the path to the serial port
    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    /// Set the baud rate in symbols-per-second
    pub fn baud_rate(mut self, baud_rate: usize) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    /// Set the number of bits used to represent a character sent on the line
    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    /// Set the type of signalling to use for controlling data transfer
    pub fn flow_control(mut self, flow_control: FlowControl) -> Self {
        self.flow_control = flow_control;
        self
    }

    /// Set the type of parity to use for error checking
    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// Set the number of bits to use to signal the end of a character
    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    /// Set the amount of time to wait to receive data before timing out
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn open(&self) -> serialport::Result<Box<dyn SerialPort>> {
        serialport::new(self.path.clone(), self.baud_rate as u32)
            .timeout(self.timeout)
            .data_bits(self.data_bits)
            .parity(self.parity)
            .flow_control(self.flow_control)
            .stop_bits(self.stop_bits)
            .open()
    }
}

pub fn get_serial_devices() -> Result<Vec<String>, Error> {
    let ports = serialport::available_ports().expect("Getting all available ports");
    let ports: Vec<String> = ports.iter().map(|p| p.port_name.clone()).collect();
    Ok(ports)
}
