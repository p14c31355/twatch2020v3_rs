pub type EspSharedBusI2c0<'a> = shared_bus::I2cProxy<'a, std::sync::Mutex<EspI2c0>>;

use crate::gpio::Output;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::{gpio, prelude::*};
use log::*;
use std::time;
use esp_idf_hal::spi;
use mipidsi::*;

pub type EspI2c0 = esp_idf_hal::i2c::Master<
    gpio::Gpio35<gpio::Output>,
    gpio::Gpio21<gpio::Output>,
    gpio::Gpio22<gpio::Output>,
>;

pub type EspSpi2InterfaceNoCS = SPIInterfaceNoCS<
    spi::Master<
        spi::SPI3,
        gpio::Gpio18<Output>,
        gpio::Gpio19<Output>,
        gpio::Gpio23<Output>,
        gpio::Gpio5<Output>,
    >,
    gpio::Gpio27<Output>,
>;

pub struct TwatchDisplay {
    pub display: Display<EspSpi2InterfaceNoCS, mipidsi::NoPin, mipidsi::models::ST7789>,
    pub backlight: Backlight,
    pub framebuffer: &'static mut FrameBuf<Rgb565, 57600_usize>
}
/*
pub struct Hal {
    pub motor: gpio::Gpio4<Output>, // define Vibration motor
}

pub struct Pmu<'a> {
    axp20x: axp20x::Axpxx<EspSharedBusI2c0<'a>>,
}
*/

impl Twatch {
    pub fn new(peripherals: Peripherals) -> Self {
        let pins = peripherals.pins;
        /*
        let motor = pins
            .gpio4
            .into_output()
            .expect("Unable to set gpio4 to output");
        */
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum State {
    On,
    Off,
}
/*
impl From<State> for axp20x::PowerState {
    fn from(state: State) -> Self {
        match state {
            State::On => axp20x::PowerState::On,
            State::Off => axp20x::PowerState::Off,
        }
    }
}
*/

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

/*
pub fn button_to_motor(&mut self) -> Result<()> {
    Ok(())
}
*/
