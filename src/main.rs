use anyhow::Result;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::prelude::*;
use esp_idf_hal::peripherals::Peripherals;
use std::thread;
use std::time::Duration;

use axp20x::{Axpxx, Power, PowerState};
use esp_idf_hal::delay::FreeRtos;

fn main() -> Result<()> {
    // ESP-IDF パッチをリンク
    esp_idf_sys::link_patches();
    let p = Peripherals::take().unwrap();

    // I2C 初期化
    let i2c_cfg = I2cConfig::new().baudrate(50_000.Hz());
    let mut i2c = I2cDriver::new(p.i2c0, p.pins.gpio21, p.pins.gpio22, &i2c_cfg)?;

    // AXP20X インスタンス作成
    let mut axp = Axpxx::new(&mut i2c);
    axp.init().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    let mut delay = FreeRtos;

    // DC-DC1 出力（例: バックライト）を有効化
    axp.set_power_output(Power::Ldo2, PowerState::On, &mut delay)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    // 出力電圧を 3.3V に設定 (このバージョンでは直接設定メソッドが見つからないためコメントアウト)
    // axp.set_dcdc1_voltage(3300)?;

    // バックライトを点滅させるループ
    loop {
        // ON
        axp.set_power_output(Power::Ldo2, PowerState::On, &mut delay)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        println!("バックライト ON");
        thread::sleep(Duration::from_millis(500));

        // OFF
        axp.set_power_output(Power::Ldo2, PowerState::Off, &mut delay)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        println!("バックライト OFF");
        thread::sleep(Duration::from_millis(500));
    }
}
