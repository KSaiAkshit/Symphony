use serde::{Deserialize, Serialize};
use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::time::Duration;
use thiserror::Error;

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
    pub name: String,
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
            devices: vec![Device::default()],
            labels: vec![vec!["".to_string()]],
            number_of_plots: vec![1],
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_millis(10),
        }
    }
}

pub fn get_serial_devices() -> Result<Vec<String>, Error> {
    let ports = serialport::available_ports().expect("Getting all available ports");
    let ports: Vec<String> = ports.iter().map(|p| p.port_name.clone()).collect();
    Ok(ports)
}
