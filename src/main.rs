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

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut delay = FreeRtos;
    let i2c_cfg = I2cConfig::new().baudrate(400_000.Hz());

    let i2c_hal_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_cfg,
    )?;

    let mut i2c_manager = I2cManager::new(i2c_hal_driver);

    let mut display = TwatchDisplay::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
    )?;

    let mut app = App::new(&mut i2c_manager, &mut display);

    app.run(&mut delay)?;
    Ok(())
}