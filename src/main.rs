use std::time::Instant;
use std::time::Duration;

use button_driver::{Button, ButtonConfig};
use esp_idf_hal::gpio;
use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_sys::EspError;
use log::info;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;

    let mut button: InputPin = gpio::Pull;
    let mut button1 = pin.is_set_low();
    Ok(())
}