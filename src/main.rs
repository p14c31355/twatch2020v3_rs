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

    let mut i2c = esp_idf_hal::i2c::I2cDriver::new(
        i2c0,
        gpio21,
        gpio22,
        &esp_idf_hal::i2c::I2cConfig::new().baudrate(esp_idf_hal::units::Hertz(400_000)),
    )?;
    let mut power = PowerManager::new(&mut i2c)?;

    let mut buffer = [0_u8; 240 * 240 * 2]; // 240x240 pixels, 2 bytes per pixel (Rgb565)
    let mut display = init_display(p, &mut buffer, &mut delay)?;

    loop {
        power.backlight_on(&mut delay)?;
        println!("ON");
        thread::sleep(Duration::from_millis(500));

        power.backlight_off(&mut delay)?;
        println!("OFF");
        thread::sleep(Duration::from_millis(500));
    }
}
