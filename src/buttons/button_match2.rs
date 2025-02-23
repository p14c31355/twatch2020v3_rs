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

    let mut last_state = if pin.is_low() { State::On } else { State::Off };
    let debounce_delay = Duration::from_millis(50);

    loop {
        let current_state = if pin.is_low() { State::On } else { State::Off };

        match current_state {
            state if state == last_state => {
                // 状態が変わっていない場合は何もしない
                thread::sleep(Duration::from_millis(10));
            }
            state => {
                // 状態が変わった場合
                thread::sleep(debounce_delay);
                let debounced_state = if pin.is_low() { State::On } else { State::Off };
                if debounced_state == state {
                    match state {
                        State::On => info!("HelloButton!"),
                        State::Off => {}
                    }
                    last_state = state;
                }
            }
        }
    }
}