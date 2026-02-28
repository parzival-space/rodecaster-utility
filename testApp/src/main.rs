use std::thread::sleep;
use std::time::Duration;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use rodecaster_usb::{DeviceManager, RodeCasterDevice};

fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();
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

    let device = DeviceManager::open_rodecaster_device(1, 14)
        .expect("Failed to open Rodecaster Pro II");

    let RodeCasterDevice::RodeCasterProII(mut rodecaster) = device else {
        panic!("Failed to open Rodecaster Pro II");
    };

    info!("Opened rodecaster Pro II");
    info!("RodeCaster Pro II connected at bus {} and address {}", rodecaster.get_bus_number(), rodecaster.get_address());
    info!("Vendor ID: {:04x}, Product ID: {:04x}", rodecaster.get_vendor_id(), rodecaster.get_product_id());
    info!("Manufacturer: {}, Product: {}",
        rodecaster.get_manufacturer_string().expect("Failed to read manufacturer string"),
        rodecaster.get_product_string().expect("Failed to read product string")
    );
    info!("Serial Number: {}",
        rodecaster.get_serial_number_string().expect("Failed to read serial number string")
    );

    // sleep(Duration::from_secs(1));
    // rodecaster.write_interrupt(&[0x03, 0x04, 0x00, 0x00, 0x00, 0xAD, 0x10, 0xA7, 0xB0]);
    // info!("Sent init message");
    //
    // sleep(Duration::from_secs(1));
    // let mut continue_reading = true;
    // while continue_reading {
    //     let data = rodecaster.read_interrupt();
    //     continue_reading = data.is_ok();
    //
    //     if (data.is_err()) {
    //         info!("Failed to read from device, maybe it was disconnected? Error: {:?}", data.err());
    //         break;
    //     } else {
    //         info!("Received data: {:?}", data.unwrap());
    //     }
    // }
}