// src/main.rs
mod app;
mod drivers;
use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::prelude::*;
use drivers::{axp::PowerManager, display::TwatchDisplay, touch::Touch};
use app::App;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut delay = FreeRtos;
    let i2c_cfg = I2cConfig::new().baudrate(400_000.Hz());
    let i2c_driver = esp_idf_hal::i2c::I2cDriver::new(
    peripherals.i2c0,
    peripherals.pins.gpio21,
    peripherals.pins.gpio22,
    &i2c_cfg,
)?;
    // Create a mutable reference to i2c_driver for PowerManager and Touch
    let mut i2c_driver_ref = i2c_driver;
    let mut power = PowerManager::new(&mut i2c_driver_ref)?;
    let mut touch = Touch::new_with_ref(&mut i2c_driver_ref)?;

    let mut display_buffer = [0_u8; 240 * 240 * 2];
let display = TwatchDisplay::new(
    peripherals.spi2,
    peripherals.pins.gpio18,
    peripherals.pins.gpio23,
    peripherals.pins.gpio5,
    peripherals.pins.gpio27,
    peripherals.pins.gpio33,
    &mut display_buffer,
)?;

    let mut app = App::new_with_i2c(i2c_driver, display);

    app.run(&mut delay)?;
    Ok(())
}
