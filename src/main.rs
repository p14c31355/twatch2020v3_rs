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
    let i2c_cfg = I2cConfig::new().baudrate(100_000.Hz());
    let mut i2c = I2cDriver::new(p.i2c0, p.pins.gpio21, p.pins.gpio22, &i2c_cfg)?;

    let axp_addr = 0x35u8;

    loop {
        // LDO2 (REG0x23)
        let mut buf = [0u8; 1];
        i2c.read(axp_addr, &mut buf, 0x23)?;
        let ldo2_setting = buf[0];
        let ldo2_mv = 700 + (ldo2_setting as u32) * 25;

        // LDO3 (REG0x24)
        i2c.read(axp_addr, &mut buf, 0x24)?;
        let ldo3_setting = buf[0];
        let ldo3_mv = 700 + (ldo3_setting as u32) * 25;

        // DCDC1 (REG0x26)
        i2c.read(axp_addr, &mut buf, 0x26)?;
        let dcdc1_setting = buf[0];
        let dcdc1_mv = 700 + (dcdc1_setting as u32) * 25;

        println!(
            "LDO2: 0x{:02X} (~{} mV), LDO3: 0x{:02X} (~{} mV), DCDC1: 0x{:02X} (~{} mV)",
            ldo2_setting, ldo2_mv,
            ldo3_setting, ldo3_mv,
            dcdc1_setting, dcdc1_mv
        );

        thread::sleep(Duration::from_secs(1));
    }
}
