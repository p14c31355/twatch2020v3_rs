// main.rs
mod app;
mod drivers;
mod manager;

use anyhow::Result;
use app::App;
use drivers::axp::PowerManager;
use drivers::display::TwatchDisplay;
use drivers::touch::Touch;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use manager::I2cManager;

fn init_watchdog() {
    unsafe {
        esp_idf_sys::esp_task_wdt_add(core::ptr::null_mut());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Debug);

    init_watchdog();

    let peripherals = Peripherals::take().unwrap();
    let mut display_buffer = Box::new([0u8; 240 * 240 * 2]);

    // I2C0: Power
    let i2c0_cfg = I2cConfig::new().baudrate(100_000.Hz());
    let i2c0_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c0_cfg,
    )?;
    let mut i2c0_manager = I2cManager::new(i2c0_driver);

    let _ = scan_i2c(&mut i2c0_manager);

    // I2C1: Touch
    let i2c1_cfg = I2cConfig::new().baudrate(400_000.Hz());
    let i2c1_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c1,
        peripherals.pins.gpio23,
        peripherals.pins.gpio32,
        &i2c1_cfg,
    )?;
    let i2c1_manager = I2cManager::new(i2c1_driver);

    // PowerManager init
    let mut power = PowerManager::new(i2c0_manager.clone())?;
    power.set_backlight(true)?;

    FreeRtos::delay_ms(50);
    let display = TwatchDisplay::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio19,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
        display_buffer.as_mut(),
    )?;

    let touch = Touch::new(i2c1_manager)?;

    let mut app = App::new(i2c0_manager, display, power, touch)?;
    app.run()?;
    Ok(())
}

fn scan_i2c<T: embedded_hal::i2c::I2c>(i2c: &mut T) -> Result<(), anyhow::Error> {
    for addr in 0x03..=0x77 {
    let mut buf = [0u8; 1];
    if i2c.write_read(addr, &[0x00], &mut buf).is_ok() {
        println!("Found device at 0x{:02X}", addr);
    }
}

    Ok(())
}