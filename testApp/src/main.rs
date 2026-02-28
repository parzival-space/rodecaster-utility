use rodecaster_usb::{DeviceManager, RodeCasterDevice};

fn main() {
    // CombinedLogger::init(
    //     vec![
    //         TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
    //     ]
    // ).unwrap();
    //
    // let deviceInfo = list_devices().wait()?
    //     .find(|dev| dev.vendor_id() == VID_RODE && dev.product_id() == PID_RODECASTER_PRO_II_EXTENDED)
    //     .ok_or("Rodecaster Pro II not found").unwrap();
    //
    // let mut rodecaster = RodecasterProII::open(deviceInfo)
    //     .expect("Failed to open Rodecaster Pro II");
    //
    // // log all input reads
    // loop {
    //     let mut buf = rodecaster.read_interrupt()
    //         .expect("Failed to read from Rodecaster Pro II");
    //     debug!("Received data: {:?}", buf);
    // }

    let device = DeviceManager::open_rodecaster_device(1, 5)
        .expect("Failed to open Rodecaster Pro II");

    let RodeCasterDevice::RodeCasterProII(rodecaster) = device else {
        panic!("Failed to open Rodecaster Pro II");
    };

    println!("Opened rodecaster Pro II");
    println!("RodeCaster Pro II connected at bus {} and address {}", rodecaster.get_bus_number(), rodecaster.get_address());
    println!("Vendor ID: {:04x}, Product ID: {:04x}", rodecaster.get_vendor_id(), rodecaster.get_product_id());
    println!("Manufacturer: {}, Product: {}",
        rodecaster.get_manufacturer_string().expect("Failed to read manufacturer string"),
        rodecaster.get_product_string().expect("Failed to read product string")
    );
    println!("Serial Number: {}",
        rodecaster.get_serial_number_string().expect("Failed to read serial number string")
    );
}