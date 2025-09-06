mod app;
mod drivers;

use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::delay::FreeRtos;
use std::thread;
use std::time::Duration;

use drivers::{axp::PowerManager, display::init_display};

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    let p = Peripherals::take().unwrap();
    let mut delay = FreeRtos;

    let i2c0 = p.i2c0;
    let gpio21 = p.pins.gpio21;
    let gpio22 = p.pins.gpio22;

    let i2c = esp_idf_hal::i2c::I2cDriver::new(
        i2c0,
        gpio21,
        gpio22,
        &esp_idf_hal::i2c::I2cConfig::new().baudrate(esp_idf_hal::units::Hertz(400_000)),
    )?;
    let mut power = PowerManager::new(i2c)?;

    let spi2 = p.spi2;
    let gpio18 = p.pins.gpio18;
    let gpio23 = p.pins.gpio23;
    let gpio5 = p.pins.gpio5;
    let gpio27 = p.pins.gpio27;
    let gpio33 = p.pins.gpio33;

    let mut buffer = [0_u8; 240 * 240 * 2]; // 240x240 pixels, 2 bytes per pixel (Rgb565)
    let _display = init_display(
        spi2, gpio18, gpio23, gpio5, gpio27, gpio33, &mut buffer
    )?;

    loop {
        power.backlight_on(&mut delay)?;
        println!("ON");
        thread::sleep(Duration::from_millis(500));

        power.backlight_off(&mut delay)?;
        println!("OFF");
        thread::sleep(Duration::from_millis(500));
    }
}
