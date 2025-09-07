// main.rs
mod app;
mod manager;
mod drivers;

use anyhow::Result;
use esp_idf_hal::prelude::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::i2c::I2cConfig;
use drivers::display::TwatchDisplay;
use manager::I2cManager;
use app::App;
use drivers::axp::PowerManager;
use drivers::touch::Touch;

use std::panic;
use esp_idf_hal::task::watchdog::{TWDT, TWDTDriver, TWDTConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ESP-IDF 初期化
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Debug);

    // println!("WDT timeout: {}", esp_idf_sys::CONFIG_ESP_TASK_WDT_TIMEOUT_S);

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

    // TWDT 初期化（全タスク監視、10秒タイムアウトなど）
    let twdt_config = TWDTConfig {
        duration: std::time::Duration::from_secs(10),
        ..Default::default()
    };
    let mut twdt_driver = TWDTDriver::new(peripherals.twdt, &twdt_config)?;

    // I2C 初期化
    let i2c_cfg = I2cConfig::new().baudrate(400_000.Hz());
    let i2c_hal_driver = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_cfg,
    )?;
    let i2c_manager = I2cManager::new(i2c_hal_driver);

    // Display 初期化
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

    // Power & Touch
    let power = PowerManager::new()?;
    let touch = Touch::new()?;

    // アプリ初期化
    let mut app = App::new(i2c_manager, display, power, touch, twdt_driver)?;

    app.run()?; // main loop
    Ok(())
}
