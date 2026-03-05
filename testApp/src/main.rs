use std::fs;
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

    let mut device_manager = DeviceManager::new()
        .expect("Failed to initialize Device Manager");

    let device_serial = device_manager.list_devices()
        .expect("Failed to list devices")
        .first()
        .expect("No RODECaster Pro II devices found. Please connect a device and try again.")
        .to_owned();

    let device = device_manager.open_rodecaster_device(&device_serial)
        .expect("Failed to open RODECaster device");

    let RodeCasterDevice::RodeCasterProII(mut rodecaster) = device else {
        panic!("Failed to open Rodecaster Pro II");
    };

    info!("Opened rodecaster Pro II");
    info!("Vendor ID: {:04x}, Product ID: {:04x}", rodecaster.get_vendor_id(), rodecaster.get_product_id());
    info!("Manufacturer: {}, Product: {}",
        rodecaster.get_manufacturer_string().expect("Failed to read manufacturer string"),
        rodecaster.get_product_string().expect("Failed to read product string")
    );
    info!("Serial Number: {}",
        rodecaster.get_serial_number_string().expect("Failed to read serial number string")
    );

    sleep(Duration::from_secs(2));
    let mut continue_reading = true;
    let mut index: u16 = 0;
    while continue_reading {
        let data = rodecaster.read();
        continue_reading = data.is_ok();

        if (data.is_err()) {
            info!("Failed to read from device, maybe it was disconnected? Error: {:?}", data.err());
            break;
        } else {
            // info!("Read data (index {})", index);
            // // write bytes into file init_XX.bin, notice the double digits in the file name, so that the files are sorted by index when listed in a directory
            //
            // fs::write(format!("init_{:02}.bin", index), data.unwrap())
            //     .expect("Failed to write data to file");
            // index += 1;
        }
    }
}