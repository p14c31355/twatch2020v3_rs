use esp_idf_hal::i2c::{I2cDriver, I2cConfig};
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // ESP-IDF std 環境初期化
    esp_idf_sys::link_patches();

    let p = Peripherals::take().unwrap();

    // --- I2C 初期化 ---
    let i2c_cfg = I2cConfig::new().baudrate(100_000.Hz());
    let mut i2c = I2cDriver::new(p.i2c0, p.pins.gpio21, p.pins.gpio22, &i2c_cfg)?;

    let axp_addr = 0x35u8;

    // --- AXP202 全レジスタ読み取り ---
    println!("Reading AXP202 registers:");
    for reg in 0x00..=0xFF {
        let mut buf = [0u8; 1];
        if i2c.write_read(axp_addr, &[reg], &mut buf, 1000).is_ok() {
            println!("REG0x{:02X} = 0x{:02X}", reg, buf[0]);
        }
    }

    // --- LDO2 / DCDC1 / DCDC3 を ON ---
    // REG0x12: power control
    let mut reg12 = [0u8; 1];
    i2c.write_read(axp_addr, &[0x12], &mut reg12, 1000)?;
    reg12[0] |= 0x1F; // LDO2/DCDC1/DCDC3 ON + 他必要なビット
    i2c.write(axp_addr, &[0x12, reg12[0]], 1000)?;
    println!("REG0x12 updated to 0x{:02X}", reg12[0]);

    // --- GPIO12 (バックライト) PWM 最大点灯 ---
    let timer = LedcTimerDriver::new(p.ledc.timer0, &TimerConfig::new().frequency(25.kHz().into()))?;
    let mut channel = LedcDriver::new(p.ledc.channel0, &timer, p.pins.gpio12)?;
    channel.set_duty(channel.get_max_duty())?;

    println!("Backlight ON (GPIO12 PWM max)");

    // --- 1秒おきに AX202 REG0x12 を監視 ---
    loop {
        i2c.write_read(axp_addr, &[0x12], &mut reg12, 1000)?;
        println!("REG0x12 current value: 0x{:02X}", reg12[0]);
        thread::sleep(Duration::from_secs(1));
    }
}
