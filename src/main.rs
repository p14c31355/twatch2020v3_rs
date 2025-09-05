use anyhow::Result;
use esp_idf_hal::{
    i2c::{I2cConfig, I2cDriver},
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
    peripherals::Peripherals,
    prelude::*,
};
use std::thread;
use std::time::Duration;

/// AXP202 I2C アドレス
const AXP202_ADDR: u8 = 0x35;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    let p = Peripherals::take().unwrap();

    // --- I2C 初期化 ---
    let i2c_cfg = I2cConfig::new().baudrate(100.kHz().into());
    let mut i2c = I2cDriver::new(p.i2c0, p.pins.gpio21, p.pins.gpio22, &i2c_cfg)?;

    // --- AXP202 レールを安全に順番に ON ---
    // 0x12 : DCDC1/2/3, LDO2/3
    let mut reg12 = i2c_read_reg(&mut i2c, 0x12)?;
    println!("AXP202 REG0x12 current value: 0x{:02X}", reg12);
    reg12 |= 0b00011111; // DCDC1/2/3 + LDO2/3 ON
    i2c_write_reg(&mut i2c, 0x12, reg12)?;
    println!("AXP202 REG0x12 updated to 0x{:02X}", reg12);
    thread::sleep(Duration::from_millis(50));

    // 0x10 : LDO1 (RTC/CPU 用)
    let mut reg10 = i2c_read_reg(&mut i2c, 0x10)?;
    println!("AXP202 REG0x10 current value: 0x{:02X}", reg10);
    reg10 |= 0b00000001; // LDO1 ON
    i2c_write_reg(&mut i2c, 0x10, reg10)?;
    println!("AXP202 REG0x10 updated to 0x{:02X}", reg10);
    thread::sleep(Duration::from_millis(50));

    // --- バックライト点灯 (GPIO12) ---
    let timer = LedcTimerDriver::new(p.ledc.timer0, &TimerConfig::new().frequency(25.kHz().into()))?;
    let mut channel = LedcDriver::new(p.ledc.channel0, &timer, p.pins.gpio12)?;
    channel.set_duty(channel.get_max_duty())?;
    println!("Backlight ON");

    // --- メインループ ---
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

/// I2C レジスタ読み取り
fn i2c_read_reg(i2c: &mut I2cDriver<'_>, reg: u8) -> Result<u8> {
    let mut buf = [0u8];
    i2c.write_read(AXP202_ADDR, &[reg], &mut buf, 100)?;
    Ok(buf[0])
}

/// I2C レジスタ書き込み
fn i2c_write_reg(i2c: &mut I2cDriver<'_>, reg: u8, val: u8) -> Result<()> {
    i2c.write(AXP202_ADDR, &[reg, val], 100)?;
    Ok(())
}
