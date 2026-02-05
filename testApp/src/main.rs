use std::io::{Error, Read, Write};
use std::thread::sleep;
use std::time::Duration;
use nusb::{list_devices, DeviceInfo, MaybeFuture};
use nusb::descriptors::language_id::US_ENGLISH;
use nusb::transfer::{In, Interrupt, Out};
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use log::debug;
use rodecaster_usb::{PID_RODECASTER_PRO_II_EXTENDED, VID_RODE};
use rodecaster_usb::rodecaster_pro_ii::RodecasterProII;

fn main() -> Result<(), Error> {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        ]
    ).unwrap();

    let deviceInfo = list_devices().wait()?
        .find(|dev| dev.vendor_id() == VID_RODE && dev.product_id() == PID_RODECASTER_PRO_II_EXTENDED)
        .ok_or("Rodecaster Pro II not found").unwrap();

    let mut rodecaster = RodecasterProII::open(deviceInfo)
        .expect("Failed to open Rodecaster Pro II");

    // log all input reads
    loop {
        let mut buf = rodecaster.read_interrupt()
            .expect("Failed to read from Rodecaster Pro II");
        debug!("Received data: {:?}", buf);
    }

    Ok(())
}