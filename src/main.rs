// src/main.rs
mod app;
mod drivers;

use anyhow::Result;
use esp_idf_hal::prelude::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use drivers::display::TwatchDisplay;
use app::App;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut delay = FreeRtos;

    let i2c_cfg = I2cConfig::new().baudrate(400_000.Hz());
    let i2c_driver = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_cfg,
    )?;

    let i2c_driver: &'static mut I2cDriver = Box::leak(Box::new(i2c_driver));

    let mut display_buffer = Box::leak(Box::new([0_u8; 240 * 240 * 2]));

    let display = TwatchDisplay::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
        &mut display_buffer[..],
    )?;

    let mut app = App::new(i2c_driver, display);
    app.run(&mut delay)?;
    Ok(())
}
