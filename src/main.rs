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

    let mut last_state = pin.is_low().map(|low| if low { State::On } else { State::Off })?;
    let debounce_delay = Duration::from_millis(50);

    loop {
        let current_state = pin.is_low().map(|low| if low { State::On } else { State::Off })?;

        if current_state != last_state {
            thread::sleep(debounce_delay);
            let debounced_state = pin.is_low().map(|low| if low { State::On } else { State::Off })?;
            if debounced_state == current_state {
                if current_state == State::On {
                    info!("HelloButton!");
                }
                last_state = current_state;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}