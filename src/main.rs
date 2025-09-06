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

    // Create a single instance of I2C driver to be used by both Power and Touch
    let mut i2c_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_cfg,
    )?;

    // Create a new I2C bus to be used for the touch driver, preventing multiple mutable borrows
    let mut touch_i2c_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c1, // Use a different I2C peripheral, or restructure to pass a shared reference if possible
        peripherals.pins.gpio32,
        peripherals.pins.gpio33,
        &i2c_cfg,
    )?;

    // Create the power and touch managers
    let power = PowerManager::new(&mut i2c_driver)?;
    let touch = Touch::new_with_ref(&mut touch_i2c_driver)?;

    // The display buffer must be owned by the App struct.
    let mut display_buffer = Box::new([0_u8; 240 * 240 * 2]);
    let display = TwatchDisplay::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
        &mut *display_buffer,
    )?;
    
    // The App struct needs to own the drivers to extend their lifetime.
    let mut app = App::new(power, display, touch);

    app.run(&mut delay)?;
    Ok(())
}