pub type EspSharedBusI2c0<'a> = shared_bus::I2cProxy<
    'a,
    std::sync::Mutex<EspI2c0>,
>;

use std::time;
use esp_idf_hal::prelude::*;

pub type EspI2c0 =
    esp_idf_hal::i2c::Master<i2c::I2C0, gpio::Gpio21<gpio::Output>, gpio::Gpio22<gpio::Output>>;

use crate::types::EspSharedBusI2c0;

pub struct Hal {
    pub motor: gpio::Gpio4<Output> // define Vibration motor
}

pub struct Pmu<'a> {
    axp20x: axp20x::Axpxx<EspSharedBusI2c0<'a>>,
}

impl Twatch {
    pub fn new(peripherals: Peripherals) -> Self {
        let pins = peripherals.pins;
        let motor = pins.gpio4.into_output().expect("Unable to set gpio4 to output");

    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum State {
    On,
    Off,
}

impl From<State> for axp20x::PowerState {
    fn from(state: State) -> Self {
        match state {
            State::On => axp20x::PowerState::On,
            State::Off => axp20x::PowerState::Off,
        }
    }
}

fn main() {
    // 今の時刻を取得
    let now = time::Instant::now();

    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");
    // 最初の時刻からの経過時間を表示
    println!("{:?}", now.elapsed());
}

pub fn button_to_motor(&mut self) -> Result<()> {
    Ok(())

    
}