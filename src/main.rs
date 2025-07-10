use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig},
    gpio::PinDriver,
    peripherals::Peripherals,
    prelude::FromValueType, // kHz() メソッドのために必要
};
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// axp192 v0.1.0 に ChargerState が存在する場合、必要に応じてインポート
// use axp192::ChargerState; 
use axp192::Axp192;
// axp192 v0.1.0 に irq モジュールや IRQManager トレイトがない可能性があるので、削除またはコメントアウト
// use axp192::irq::IRQManager; 

// embedded-hal::blocking::i2c と DelayMs は v0.1.0 で必要
use embedded_hal::blocking::i2c::{Read, Write, WriteRead}; 
use embedded_hal::blocking::delay::DelayMs; 
use esp_idf_svc::hal::delay::FreeRtos;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mut delay = FreeRtos; // v0.1.0 の Axp192::new() は DelayMs を必要とする可能性が高い

    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();

    // --- AXP192 (I2C) の初期化 ---
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()), // FromValueType がスコープにある
    )?;

    // AXP192 ドライバーインスタンスを作成
    // v0.1.0 の Axp192::new() は I2C と DelayMs の両方を引数に取り、Result を返すことが一般的
    let mut axp = Axp192::new(i2c, delay)?; 
    info!("AXP192 initialized!");

    // --- AXP192 Specific Configuration ---
    // v0.1.0 の API に合わせてメソッド名を再確認する必要があります。
    // enable_pek_irq, clear_all_irqs がAxp192構造体に直接実装されているか確認。
    // もしそれでも "method not found" が出る場合、v0.1.0 のドキュメントを再確認するか、
    // T-Watch 2020 v3 の他の Rust サンプルで AXP192 の割り込みをどう扱っているか確認してください。
    // 一部の機能は特定のバージョンでしか提供されないか、名前が異なることがあります。
    axp.enable_pek_irq()?;
    axp.clear_all_irqs()?;

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
                info!("Button Pressed!");
                *pressed = false;
                thread::sleep(Duration::from_millis(50));
            }
        }
        thread::sleep(Duration::from_millis(100));

        // AXP192からのボタン状態のポーリング
        // v0.1.0 の API に合わせてメソッド名を再確認。
        // get_all_irqs と clear_pek_irq が Axp192 構造体に直接実装されているか確認。
        if let Ok(irq_status) = axp.get_all_irqs() {
            if irq_status.is_pek_short_press() {
                info!("AXP192 PEK Short Press detected!");
                axp.clear_pek_irq()?;
            }
        }
    }
}