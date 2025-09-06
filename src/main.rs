// main.rs
mod app;
mod manager;
mod drivers;

use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::prelude::*;
use drivers::display::TwatchDisplay;
use manager::I2cManager;
use app::App;
use drivers::axp::PowerManager;
use drivers::touch::Touch;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let i2c_cfg = I2cConfig::new().baudrate(400_000.Hz());

    let i2c_hal_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_cfg,
    )?;

    let i2c_manager = I2cManager::new(i2c_hal_driver);

    let mut display_buffer = [0_u8; 240 * 240 * 2];
    let display = TwatchDisplay::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
        display_buffer.as_mut(),
    )?;

    let power = PowerManager::new()?;
    let touch = Touch::new()?;

    let mut app = App::new(i2c_manager, display, power, touch);

    app.run(&mut FreeRtos)?;
    Ok(())
}