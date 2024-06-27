use eframe::{run_native, NativeOptions};
use symphony::gui::Symphony;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting App");

    let options = NativeOptions::default();
    run_native(
        "MyApp",
        options,
        Box::new(|_cc| Ok(Box::<Symphony>::new(Symphony::new()))),
    )
    .expect("Starting app from here");
    Ok(())
}

// fn main() {
//     let (tx, rx) = std::sync::mpsc::channel::<serialib::Packet>();
//     let start = std::time::Instant::now();
//     let mut port = std::io::BufReader::new(
//         serialport::new("/dev/pts/4", 115200)
//             .timeout(std::time::Duration::from_nanos(1))
//             .open()
//             .unwrap(),
//     );
//     let read_thread = std::thread::spawn(move || loop {
//         serialib::perform_reads(&mut port, &tx, start);
//     });

//     let print_thread = std::thread::spawn(move || loop {
//         let read = rx.recv().unwrap();
//         println!("{}", read.payload);
//     });

//     read_thread.join().unwrap();
//     print_thread.join().unwrap();
// }
