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
    let p = Peripherals::take().unwrap();
    let pin = PinDriver::input(p.pins.gpio35)?;
    let mut last = if pin.is_low() { State::On } else { State::Off };
    loop {
        let cur = if pin.is_low() { State::On } else { State::Off };
        if cur != last {
            thread::sleep(Duration::from_millis(1000));
            if cur == if pin.is_low() { State::On } else { State::Off } {
                if cur == State::On { info!("HelloButton!"); }
                last = cur;
            }
        }
        thread::sleep(Duration::from_millis(1000));
    }
}