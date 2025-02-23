use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_sys::EspError; // ?を使用するために必要
use esp_idf_svc::log::*;
use log::*;
use std::time::Duration;
use std::thread;
use std::time::Instant;

use button_driver::{Button, ButtonConfig};

fn main() -> Result<(), EspError> {
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;

    let mut button = Button::<_, Instant>::new(pin, ButtonConfig::default());

    loop {
        button.tick();

        if let Some(dur) = button.held_time() {
            info!("Total holding time {:?}", dur);

            if button.is_clicked() {
                info!("Clicked + held");
            } else if button.is_double_clicked() {
                info!("Double clicked + held");
            } else if button.holds() == 2 && button.clicks() > 0 {
                info!("Held twice with {} clicks", button.clicks());
            } else if button.holds() == 2 {
                info!("Held twice");
            }
        } else {
            if button.is_clicked() {
                info!("Click");
            } else if button.is_double_clicked() {
                info!("Double click");
            } else if button.is_triple_clicked() {
                info!("Triple click");
            } else if let Some(dur) = button.current_holding_time() {
                info!("Held for {:?}", dur);
            }
        }

        button.reset();
    }
}