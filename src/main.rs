use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig},
    gpio::PinDriver,
    peripherals::Peripherals,
    prelude::FromValueType,
};
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// axp192 クレート関連のインポートはすべて削除
// use axp192::Axp192;
// use axp192::irq::IRQManager;
// use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
// use embedded_hal::blocking::delay::DelayMs;
use esp_idf_svc::hal::delay::FreeRtos;

// AXP192のI2Cアドレス
const AXP192_ADDR: u8 = 0x34; // T-Watch 2020 V3のAXP192 I2Cアドレス

// AXP192のレジスタアドレス (一般的なもの、正確なものはデータシートで要確認)
const AXP192_PEK_IRQ_EN1: u8 = 0x46; // Interrupt Enable Register 1
const AXP192_PEK_IRQ_STATUS1: u8 = 0x48; // Interrupt Status Register 1
const AXP192_PEK_SHORT_PRESS_BIT: u8 = 0b0000_0010; // PEK_SHORT_PRESS_INT_EN (BIT1)

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let _delay = FreeRtos;

    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();

    // --- AXP192 (I2C) の初期化 ---
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let mut i2c = I2cDriver::new( // i2c を mut に変更
        peripherals.i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()),
    )?;

    // AXP192 の初期設定
    // PEK_SHORT_PRESS_INT_EN (0x46 レジスタのビット1) を有効にする
    // 既存の値にOR演算でビットを立てるのが安全ですが、
    // ここでは単純に設定します。必要に応じて既存値を読み取ってからORしてください。
    // https://github.com/Xinyuan-LilyGO/T-Watch-2020/blob/master/code/T-Watch-2.0-V3-Factory/src/AIO/axp.h#L24
    // 0x46 レジスタのデフォルト値は 0x00 です。
    // PEK_SHORT_PRESS_INT_EN (bit 1) を有効にするには 0x02 を書き込みます。
    i2c.write(AXP192_ADDR, &[AXP192_PEK_IRQ_EN1, AXP192_PEK_SHORT_PRESS_BIT])?; // Enable PEK short press IRQ
    info!("AXP192 configured for PEK IRQ!");

    // 最初にすべての割り込み状態をクリアする
    i2c.write(AXP192_ADDR, &[AXP192_PEK_IRQ_STATUS1, 0xFF])?; // 0xFF を書き込むことでクリア（データシート確認要）
    // 通常は、ステータスレジスタを読み取って、その値を再度書き込むことでクリアします。
    // let mut irq_status_buf = [0u8; 1];
    // i2c.write_read(AXP192_ADDR, &[AXP192_PEK_IRQ_STATUS1], &mut irq_status_buf)?;
    // i2c.write(AXP192_ADDR, &[AXP192_PEK_IRQ_STATUS1, irq_status_buf[0]])?;
    info!("AXP192 IRQ status cleared!");


    // --- ESP32 GPIO 35 (User Button) Initialization ---
    // GPIO35はAXP192からの割り込み信号を受けるピンです。
    // T-Watch 2020 V3では、GPIO35は通常、AXP192の割り込み出力ピン（IRQピン）に接続されています。
    // AXP192のIRQピンはアクティブLOW（割り込み発生時にLOWになる）であることが多いです。
    // そのため、NegEdge（立ち下がりエッジ）で割り込みを設定します。
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    unsafe { button.subscribe(move || {
        let mut pressed = button_pressed_clone.lock().unwrap();
        *pressed = true;
    }) }?;

    loop {
        {
            let mut pressed = button_pressed.lock().unwrap();
            if *pressed {
                info!("Button Pressed (from ESP32 GPIO)!");
                *pressed = false;

                // AXP192の割り込みステータスを読み取り、クリアする
                let mut irq_status_buf = [0u8; 1];
                if i2c.write_read(AXP192_ADDR, &[AXP192_PEK_IRQ_STATUS1], &mut irq_status_buf).is_ok() {
                    let irq_status = irq_status_buf[0];
                    if (irq_status & AXP192_PEK_SHORT_PRESS_BIT) != 0 {
                        info!("AXP192 PEK Short Press detected (from I2C poll)!");
                    }
                    // 読み取ったレジスタの値をそのまま書き戻すことでクリアします (データシートの方法)
                    if i2c.write(AXP192_ADDR, &[AXP192_PEK_IRQ_STATUS1, irq_status]).is_ok() {
                        info!("AXP192 IRQ status cleared via I2C.");
                    }
                }

                thread::sleep(Duration::from_millis(50)); // Debounce
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}