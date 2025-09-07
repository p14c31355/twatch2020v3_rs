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
use std::panic;

fn feed_watchdog() {
    unsafe {
        let _ = esp_idf_sys::esp_task_wdt_add(core::ptr::null_mut());
        esp_idf_sys::esp_task_wdt_reset();
    };
}

fn feed_watchdog_during<F: FnMut()>(mut f: F, steps: u32, delay_ms: u32) {
    for _ in 0..steps {
        feed_watchdog();
        f();
        FreeRtos::delay_ms(delay_ms);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Debug);

    panic::set_hook(Box::new(|info| {
        log::error!("PANIC: {:?}", info);
        let bt = std::backtrace::Backtrace::force_capture();
        log::error!("{:?}", bt);
        println!("PANIC: {:?}", info);
        println!("{:?}", bt);
    }));

    let peripherals = Peripherals::take().unwrap();
    let mut display_buffer = Box::new([0u8; 240 * 240 * 2]);

    let mut power = PowerManager::new()?;
    feed_watchdog_during(|| {}, 5, 10);

    let i2c0_cfg = I2cConfig::new().baudrate(100_000.Hz());
    let i2c0_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c0_cfg,
    )?;
    let i2c0_manager = I2cManager::new(i2c0_driver);
    feed_watchdog_during(|| {}, 5, 10);

    let i2c1_cfg = I2cConfig::new().baudrate(400_000.Hz());
    let i2c1_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c1,
        peripherals.pins.gpio23,
        peripherals.pins.gpio32,
        &i2c1_cfg,
    )?;
    let i2c1_manager = I2cManager::new(i2c1_driver);
    feed_watchdog_during(|| {}, 5, 10);

    let display = TwatchDisplay::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio19,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
        display_buffer.as_mut(),
    )?;
    feed_watchdog_during(|| {}, 5, 10);

    let touch = Touch::new()?;
    feed_watchdog_during(|| {}, 5, 10);

    let mut app = App::new(i2c0_manager, display, power, touch);

    app.run()?;
    Ok(())
}
