// main.rs
mod app;
mod drivers;
mod manager;

use anyhow::Result;
use app::App;
use drivers::axp::PowerManager;
use drivers::display::TwatchDisplay;
use drivers::touch::Touch;
use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use manager::I2cManager;

use std::panic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ESP-IDF init
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Debug);

    // Panic hook
    panic::set_hook(Box::new(|info| {
        log::error!("PANIC: {:?}", info);
        let bt = std::backtrace::Backtrace::force_capture();
        log::error!("{:?}", bt);
        println!("PANIC: {:?}", info);
        println!("{:?}", bt);
    }));

    // Peripherals
    let peripherals = Peripherals::take().unwrap();

    // Display buf init
    let mut display_buffer = Box::new([0u8; 240 * 240 * 2]);

    // I2C init
    let i2c_cfg = I2cConfig::new().baudrate(400_000.Hz());
    let i2c_hal_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_cfg,
    )?;
    let i2c_manager = I2cManager::new(i2c_hal_driver);

    let display = TwatchDisplay::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
        display_buffer.as_mut(),
    )?;

    // Power & Touch
    let power = PowerManager::new()?;
    let touch = Touch::new()?;

    // App init
    let mut app = App::new(i2c_manager, display, power, touch);

    app.run()?; // main loop
    Ok(())
}
