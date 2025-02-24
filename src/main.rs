use esp_idf_hal::timer::{Timer, TimerDriver, config::Config};
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys::EspError;
use log::info;
use std::thread;
use std::time::Duration;
use async_button::*;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;
    let mut button = Button::new(pin, ButtonConfig::default());

    loop {
        match button.update().await {
            ButtonEvent::ShortPress { count } => {info!("HelloButton!")},
            ButtonEvent::LongPress => {info!("It hurts!")},
        }
    }
}