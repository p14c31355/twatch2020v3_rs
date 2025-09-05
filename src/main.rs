use anyhow::Ok;
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::i2c::{I2cDriver, I2cConfig};
use esp_idf_hal::prelude::*;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    let p = Peripherals::take().unwrap();

    // I2C 初期化
    let i2c_cfg = I2cConfig::new().baudrate(Hertz(100_000));
    let mut i2c = I2cDriver::new(p.i2c0, p.pins.gpio21, p.pins.gpio22, &i2c_cfg)?;

//     for addr in 0x00..=0x7F {
//     if i2c.write(addr, &[0], 1000).is_ok() {
//         println!("Found device @ 0x{:02X}", addr);
//     }
// // }
//     AXP202: LDO2/3/DCDC1 有効化
//     レジスタ0x12 のデフォルト値に 0x4D を OR
    i2c.write(0x35, &[0x12, 0x4D], 0)?;
    println!("AXP202 power rails enabled");

    // LEDC 初期化（GPIO12 がバックライト制御）
    let timer = LedcTimerDriver::new(p.ledc.timer0, &TimerConfig::new().frequency(25.kHz().into()))?;
    let mut channel = LedcDriver::new(p.ledc.channel0, &timer, p.pins.gpio12)?;

    // duty 最大で点灯
    channel.set_duty(channel.get_max_duty())?;
    println!("Backlight ON");
// Ok(())
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
