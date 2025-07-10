use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig},
    gpio::PinDriver,
    peripherals::Peripherals,
    prelude::FromValueType, // ここを追加！
};
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use axp192::Axp192;
use esp_idf_svc::hal::delay::FreeRtos;

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
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()), // FromValueType をインポートしたのでOK
    )?;

    // AXP192 ドライバーインスタンスを作成 ( ? を削除)
    let mut axp = Axp192::new(i2c); // ここを修正！
    info!("AXP192 initialized!");

    // --- AXP192 Specific Configuration ---
    // これらのメソッドが Result を返す場合、? を付け加える必要があります。
    // axp192 クレートのドキュメントで各メソッドの戻り値を確認してください。
    // v0.2.0のドキュメント: https://docs.rs/axp192/0.2.0/axp192/
    axp.enable_pek_irq()?; // enable_pek_irq は Result を返します
    axp.clear_all_irqs()?; // clear_all_irqs も Result を返します

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
        // ここも axp192 v0.2.0 の API に合わせます。
        // get_all_irqs() と clear_pek_irq() が Result を返すので ? が必要です。
        if let Ok(irq_status) = axp.get_all_irqs() {
            if irq_status.is_pek_short_press() {
                info!("AXP192 PEK Short Press detected!");
                axp.clear_pek_irq()?;
            }
        }
    }
}