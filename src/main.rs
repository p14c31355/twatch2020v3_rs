use anyhow::{Ok, Result};
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::i2c::{I2cDriver, I2cConfig};
use esp_idf_hal::prelude::*;

const AXP202_ADDR: u8 = 0x35;

/// レジスタ 0x12 ビットマスク
const LDO2_BIT: u8 = 1 << 2;
const LDO3_BIT: u8 = 1 << 3;
const DCDC1_BIT: u8 = 1 << 0;

fn main() -> Result<()> {
    // ESP-IDF std 環境パッチ
    esp_idf_sys::link_patches();

    let p = Peripherals::take().unwrap();

    // I2C 初期化
    let i2c_cfg = I2cConfig::new().baudrate(Hertz(100_000));
    let mut i2c = I2cDriver::new(p.i2c0, p.pins.gpio21, p.pins.gpio22, &i2c_cfg)?;

    // --- AXP202 レジスタ 0x12 チェック ---
    let mut buf = [0u8; 1];
    i2c.write_read(AXP202_ADDR, &[0x12], &mut buf, 0)?;
    let mut reg12 = buf[0];
    println!("AXP202 REG0x12 current value: 0x{:02X}", reg12);

    // 必要なビットをセット
    let required_bits = LDO2_BIT | LDO3_BIT | DCDC1_BIT;
    if reg12 & required_bits != required_bits {
        reg12 |= required_bits;
        i2c.write(AXP202_ADDR, &[0x12, reg12], 0)?;
        println!("AXP202 REG0x12 updated to 0x{:02X}", reg12);
    } else {
        println!("AXP202 power rails already enabled");
    }

    // --- LEDC 初期化 (バックライト GPIO12) ---
    let timer = LedcTimerDriver::new(p.ledc.timer0, &TimerConfig::new().frequency(25.kHz().into()))?;
    let mut channel = LedcDriver::new(p.ledc.channel0, &timer, p.pins.gpio12)?;

    // Duty最大でバックライト点灯
    channel.set_duty(channel.get_max_duty())?;
    println!("Backlight ON");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
