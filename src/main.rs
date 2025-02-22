use std::time::Instant;
use std::time::Duration;

use button_driver::{Button, ButtonConfig};
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::*;
use esp_idf_sys::EspError;
use log::info;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;

    let mut button = Button::<_, Instant>::new(pin, ButtonConfig::default());

    loop {
        button.tick();

        if let Some(held_dur) = button.held_time() {
            log_button_event(&button, Some(held_dur));
        } else if let Some(holding_dur) = button.current_holding_time() {
            info!("Held for {:?}", holding_dur);
        } else {
            log_button_event(&button, None);
        }

        button.reset();
    }
}

fn log_button_event(button: &Button<PinDriver, Instant>, held_dur: Option<Duration>) {
    let held_str = held_dur.map(|dur| format!(" + held {:?}", dur)).unwrap_or_default();

    if button.is_clicked() {
        info!("Click{}", held_str);
    } else if button.is_double_clicked() {
        info!("Double click{}", held_str);
    } else if button.is_triple_clicked() {
        info!("Triple click");
    } else if button.holds() == 2 {
        if button.clicks() > 0 {
            info!("Held twice with {} clicks{}", button.clicks(), held_str);
        } else {
            info!("Held twice{}", held_str);
        }
    }
}