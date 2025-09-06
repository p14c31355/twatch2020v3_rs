// src/main.rs
mod app;
mod drivers;

use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::prelude::*;

use drivers::{axp::PowerManager, display::init_display, touch::Touch};
use app::App;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut delay = FreeRtos;

    // Initialize I2C driver for AXP and Touch
    let i2c_cfg = I2cConfig::new().baudrate(400.kHz().into());
    let mut i2c_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_cfg,
    )?;

    // The I2cDriver needs to be split to be used by both PowerManager and Touch
    let mut power = PowerManager::new(&mut i2c_driver)?;

    let touch = Touch::new_with_ref(&mut i2c_driver)?;

    let mut buffer = [0_u8; 240 * 240 * 2]; // 240x240, Rgb565
    let display_spi = init_display(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
        &mut buffer
    )?;

    let display = display_spi;

    let mut app = App::new(power, display, touch);

    // -------- メインループ --------
    app.run(&mut delay)?;

    Ok(())
}
