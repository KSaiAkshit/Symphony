fn main() {
    let available_ports = serialib::get_serial_devices().expect("Should work");
    dbg!(available_ports);
}
