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
use esp_idf_svc::sys::TickType_t;

// AXP192のI2Cアドレス
const AXP192_ADDR: u8 = 0x34;
const AXP192_PEK_IRQ_EN1: u8 = 0x46;
const AXP192_PEK_IRQ_STATUS1: u8 = 0x48;
const AXP192_PEK_SHORT_PRESS_BIT: u8 = 0b0000_0010;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Application started."); // アプリケーション開始ログ

    let peripherals = Peripherals::take().unwrap();
    let _delay = FreeRtos;

    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();

    // --- AXP192 (I2C) の初期化 ---
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    info!("Initializing I2C driver...");
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()),
    )?;
    // src/main.rs の I2C driver initialized successfully. の後あたり
    info!("I2C driver initialized successfully.");

    // AXP192が起動するまで少し待つ (オプション)
    thread::sleep(Duration::from_millis(50)); // 例えば50ミリ秒

    // I2C タイムアウトのティック数
    let i2c_timeout_ticks: TickType_t = 100u32;

    info!("Configuring AXP192 IRQ enable...");
    // ... 続くコード

    // I2C タイムアウトのティック数
    let i2c_timeout_ticks: TickType_t = 100u32;

    info!("Configuring AXP192 IRQ enable...");
    match i2c.write( // ★ここをmatchに変更
        AXP192_ADDR,
        &[AXP192_PEK_IRQ_EN1, AXP192_PEK_SHORT_PRESS_BIT],
        i2c_timeout_ticks,
    ) {
        Ok(_) => {
            info!("AXP192 configured for PEK IRQ!");
        },
        Err(e) => {
            error!("Failed to configure AXP192 IRQ enable: {:?}", e); // エラー情報を出力
            return Err(e.into()); // main関数からエラーを返す
        }
    }

    match i2c.write( // ★ここをmatchに変更
        AXP192_ADDR,
        &[AXP192_PEK_IRQ_STATUS1, 0xFF],
        i2c_timeout_ticks,
    ) {
        Ok(_) => {
            info!("AXP192 IRQ status cleared!");
        },
        Err(e) => {
            error!("Failed to clear AXP192 IRQ status: {:?}", e); // エラー情報を出力
            return Err(e.into()); // main関数からエラーを返す
        }
    }


    // --- ESP32 GPIO 35 (User Button) Initialization ---
    info!("Initializing GPIO35 for button...");
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;
    // button.set_pull(esp_idf_svc::hal::gpio::Pull::Up)?; // この行を削除またはコメントアウト
    info!("GPIO35 pull-up/down implicitly handled (or not set)."); // ログも変更
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    info!("GPIO35 interrupt type set.");
    unsafe { button.subscribe(move || {
        let mut pressed = button_pressed_clone.lock().unwrap();
        *pressed = true;
        info!("GPIO35 interrupt triggered!"); // 割り込み発生時のログ
    }) }?;
    info!("GPIO35 subscribed to interrupts."); // 成功ログ

    info!("Entering main loop...");

    loop {
        {
            let mut pressed = button_pressed.lock().unwrap();
            if *pressed {
                info!("Button Pressed (from ESP32 GPIO)!");
                *pressed = false;

                // AXP192の割り込みステータスを読み取り、クリアする
                let mut irq_status_buf = [0u8; 1];
                match i2c.write_read(
                    AXP192_ADDR,
                    &[AXP192_PEK_IRQ_STATUS1],
                    &mut irq_status_buf,
                    i2c_timeout_ticks,
                ) {
                    Ok(_) => {
                        let irq_status = irq_status_buf[0];
                        if (irq_status & AXP192_PEK_SHORT_PRESS_BIT) != 0 {
                            info!("AXP192 PEK Short Press detected (from I2C poll)!");
                        }
                        match i2c.write(
                            AXP192_ADDR,
                            &[AXP192_PEK_IRQ_STATUS1, irq_status],
                            i2c_timeout_ticks,
                        ) {
                            Ok(_) => {
                                info!("AXP192 IRQ status cleared via I2C.");
                            }
                            Err(e) => {
                                error!("Failed to clear AXP192 IRQ status: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read AXP192 IRQ status: {:?}", e);
                    }
                }

                thread::sleep(Duration::from_millis(50));
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}