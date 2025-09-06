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

    // Create separate I2C drivers to avoid multiple mutable borrows
    let i2c_driver = esp_idf_hal::i2c::I2cDriver::new(
    peripherals.i2c0,
    peripherals.pins.gpio21,
    peripherals.pins.gpio22,
    &i2c_cfg,
)?;

    // Create the power and touch managers, passing in the I2C drivers
    let power = PowerManager::new(&mut i2c_driver)?;
    let touch = Touch::new(i2c_driver)?;

    // The display buffer is owned by a `Box` and its mutable reference is passed to `TwatchDisplay`.
    // The `TwatchDisplay` struct should own the buffer.
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

    // The App struct now takes ownership of the drivers
    let mut app = App::new(i2c_driver, display);

    app.run(&mut delay)?;
    Ok(())
}