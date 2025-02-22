use std::time::Instant;
use button_driver::{Button, ButtonConfig};
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys::EspError;
use log::info;

/*
impl Button for Instant {
    fn new(&self) {        

    }
}
*/

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;

    let mut button = Button::new(pin, ButtonConfig::default());

loop {
    button.tick();
     
    if button.is_clicked() {
        println!("Clicked!");
    }

    button.reset();

    }
}