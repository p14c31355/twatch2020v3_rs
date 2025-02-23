use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::*;
use esp_idf_sys::EspError;
use esp_idf_svc::log::*;
use log::*;
use std::thread;
use std::time::Duration;

#[derive(PartialEq, Clone, Copy)]
enum State {
    On,
    Off,
}

fn main() -> Result<(), EspError> {
    EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;
    let mut last_state = get_state(&pin)?;
    let debounce_delay = Duration::from_millis(50);
    let loop_delay = Duration::from_millis(10);

    loop {
        let current_state = get_state(&pin)?;

        if current_state != last_state {
            thread::sleep(debounce_delay);
            let debounced_state = get_state(&pin)?;
            if debounced_state == current_state {
                if current_state == State::On {
                    info!("HelloButton!");
                }
                last_state = current_state;
            }
        }
        thread::sleep(loop_delay);
    }
}

fn get_state(pin: &PinDriver<esp_idf_hal::gpio::Gpio35, esp_idf_hal::gpio::Input>) -> Result<State, EspError> {
    match pin.is_low() {
        Ok(true) => Ok(State::On),
        Ok(false) => Ok(State::Off),
        Err(e) => Err(e),
    }
}