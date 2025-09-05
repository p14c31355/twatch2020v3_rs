use esp_idf_hal::{
    i2c::{I2cConfig, I2cDriver},
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution},
    peripherals::Peripherals,
    prelude::*,
};
use anyhow::Result;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    let p = Peripherals::take()?;

    // T-Watch の内部I2Cバス（AXP202/RTC/タッチ等）: SDA=GPIO21, SCL=GPIO22
    let i2c_cfg = I2cConfig::new().baudrate(Hertz(100_000));
    let mut i2c = I2cDriver::new(p.i2c0, p.pins.gpio21, p.pins.gpio22, &i2c_cfg)?;
    
    println!("I2C init OK");
    // I2Cスキャン
    for addr in 0x00..=0x7F {
        if i2c.write(addr, &[0], 0).is_ok() {
            println!("Found I2C device @ 0x{:02X}", addr);
        }
    }
    
    // 1) AXP202 のアドレスを決める（0x34 or 0x35）
    let axp_addr: u8 = 0x35; // 必要なら 0x35 に変更（I2Cスキャンで要確認）

    // 2) REG 0x12 を読んで LDO2(bit2) を ON
    let reg = 0x12u8;
    let mut cur = [0u8; 1];
    i2c.write_read(axp_addr, &[reg], &mut cur, 0)?;
    let new = cur[0] | 0x04; // bit2=LDO2 enable
    i2c.write(axp_addr, &[reg, new], 0)?;

    // 3) GPIO12 に PWM（LEDC）: duty>0 で点灯
    let timer = LedcTimerDriver::new(
        p.ledc.timer0,
        &TimerConfig::new().frequency(Hertz(5_000)).resolution(Resolution::Bits8),
    )?;
    let mut bl = LedcDriver::new(
        p.ledc.channel0,
        &timer,
        p.pins.gpio12, // Backlight control pin
    )?;
    bl.set_duty(bl.get_max_duty() / 2)?; // 50% くらいで点灯確認

    // ここから先で ST7789V の初期化や描画を行う
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
