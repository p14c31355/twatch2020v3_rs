use anyhow::Result;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::prelude::*;
use esp_idf_hal::peripherals::Peripherals;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    let p = Peripherals::take().unwrap();

    // I2C 初期化
    let i2c_cfg = I2cConfig::new().baudrate(50_000.Hz());
    let mut i2c = I2cDriver::new(p.i2c0, p.pins.gpio21, p.pins.gpio22, &i2c_cfg)?;

    let axp_addr = 0x35u8;
    let mut prev_regs = [0u8; 256];

    loop {
        for reg in 0x00u8..=0xFF {
            let mut buf = [0u8; 1];
            if i2c.read(axp_addr, &mut buf, reg.into()).is_ok() {
                if buf[0] != prev_regs[reg as usize] {
                    println!("REG0x{:02X}: 0x{:02X} -> 0x{:02X}", reg, prev_regs[reg as usize], buf[0]);
                    prev_regs[reg as usize] = buf[0];
                }
            } else {
                println!("REG0x{:02X}: read error!", reg);
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
}
