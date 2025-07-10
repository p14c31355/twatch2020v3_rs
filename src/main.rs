use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig},
    gpio::PinDriver,
    peripherals::Peripherals,
    prelude::FromValueType, // kHz() のために必要
};
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::sys::TickType_t; // TickType_t は残します

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
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()),
    )?;

    // I2C タイムアウトのティック数
    // TickType_t は通常 u32 のエイリアスなので、直接 u32 値を渡す
    let i2c_timeout_ticks: TickType_t = 100u32; // ここを修正！

    // AXP192 の初期設定
    i2c.write(
        AXP192_ADDR,
        &[AXP192_PEK_IRQ_EN1, AXP192_PEK_SHORT_PRESS_BIT],
        i2c_timeout_ticks,
    )?;
    info!("AXP192 configured for PEK IRQ!");

    // 最初にすべての割り込み状態をクリアする
    i2c.write(
        AXP192_ADDR,
        &[AXP192_PEK_IRQ_STATUS1, 0xFF],
        i2c_timeout_ticks,
    )?;
    info!("AXP192 IRQ status cleared!");

    // --- ESP32 GPIO 35 (User Button) Initialization ---
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
                if i2c.write_read(
                    AXP192_ADDR,
                    &[AXP192_PEK_IRQ_STATUS1],
                    &mut irq_status_buf,
                    i2c_timeout_ticks,
                ).is_ok() {
                    let irq_status = irq_status_buf[0];
                    if (irq_status & AXP192_PEK_SHORT_PRESS_BIT) != 0 {
                        info!("AXP192 PEK Short Press detected (from I2C poll)!");
                    }
                    // 読み取ったレジスタの値をそのまま書き戻すことでクリアします
                    if i2c.write(
                        AXP192_ADDR,
                        &[AXP192_PEK_IRQ_STATUS1, irq_status],
                        i2c_timeout_ticks,
                    ).is_ok() {
                        info!("AXP192 IRQ status cleared via I2C.");
                    } else {
                        error!("Failed to clear AXP192 IRQ status.");
                    }
                } else {
                    error!("Failed to read AXP192 IRQ status.");
                }

                thread::sleep(Duration::from_millis(50));
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}