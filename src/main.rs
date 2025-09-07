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
use esp_idf_hal::delay::FreeRtos;

use std::panic;

// WDT feed
fn feed_watchdog_during<F: FnMut()>(mut f: F, steps: u32, delay_ms: u32) {
    for _ in 0..steps {
        unsafe { esp_idf_sys::esp_task_wdt_reset() };
        f();
        FreeRtos::delay_ms(delay_ms);
    }
}

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

    // Watchdog init
    unsafe {
        esp_idf_sys::esp_task_wdt_init(core::ptr::null());
        esp_idf_sys::esp_task_wdt_add(core::ptr::null_mut());
    }
    // Display buffer
    let mut display_buffer = Box::new([0u8; 240 * 240 * 2]);

    // --- Power init ---
    let mut power = PowerManager::new()?;
    feed_watchdog_during(|| {}, 5, 10); // initialize during feed

    // I2C init
    let i2c_cfg = I2cConfig::new().baudrate(400_000.Hz());
    let i2c_hal_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_cfg,
    )?;
    let i2c_manager = I2cManager::new(i2c_hal_driver);
    feed_watchdog_during(|| {}, 5, 10); // initialize during feed

    // Display init
    let display = TwatchDisplay::new(
        peripherals.spi2,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        peripherals.pins.gpio5,
        peripherals.pins.gpio27,
        peripherals.pins.gpio33,
        display_buffer.as_mut(),
    )?;
    feed_watchdog_during(|| {}, 5, 10);

    // Touch init
    let touch = Touch::new()?;
    feed_watchdog_during(|| {}, 5, 10);

    // App init
    let mut app = App::new(i2c_manager, display, power, touch);

    // Run main loop
    app.run()?;
    Ok(())
}
