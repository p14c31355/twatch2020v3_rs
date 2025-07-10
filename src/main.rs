use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig}, // I2Cのインポート
    gpio::PinDriver, // Pull は使わないので削除
    peripherals::Peripherals,
};
// use esp_idf_svc::eventloop::*; // 使わないので削除
use log::*;
use std::sync::{Arc, Mutex}; // ArcとMutexは、button_pressed のために必要なので残します。
use std::thread;
use std::time::Duration;

// axp192 をインポート
use axp192::Axp192; // ChargerState は不要

// embedded-hal の DelayMs は axp192 v0.2.0 では不要
// use embedded_hal::blocking::delay::DelayMs;
use esp_idf_svc::hal::delay::FreeRtos; // 遅延プロバイダ

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let _delay = FreeRtos; // AXP192::new() に渡す必要はないが、他の場所で使う可能性があるので残しておく

    // 割り込みハンドラの初期化 (main関数の先頭近くに移動)
    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();

    // --- AXP192 (I2C) の初期化 ---
    let sda = peripherals.pins.gpio21; // SDA pin
    let scl = peripherals.pins.gpio22; // SCL pin
    let i2c = I2cDriver::new(
        peripherals.i2c0, // Use I2C0 peripheral
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()), // 400kHzに型を明示 (u32を追加)
    )?;

    // AXP192 ドライバーインスタンスを作成 (delay を削除)
    let mut axp = Axp192::new(i2c)?; // Axp192::new() は Result を返さないので ? は不要
    info!("AXP192 initialized!");

    // --- AXP192 Specific Configuration ---
    axp.enable_pek_irq()?;
    axp.clear_all_irqs()?;

    // --- ESP32 GPIO 35 (User Button) Initialization ---
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;
    // T-Watchのボタンは外部プルアップされていると仮定し、NegEdgeが正しい
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    unsafe { button.subscribe(move || {
        let mut pressed = button_pressed_clone.lock().unwrap();
        *pressed = true;
    }) }?;

    // ... (rest of your main loop) ...
    loop {
        {
            let mut pressed = button_pressed.lock().unwrap();
            if *pressed {
                info!("Button Pressed!");
                *pressed = false;
                thread::sleep(Duration::from_millis(50)); // Debounce
            }
        }
        thread::sleep(Duration::from_millis(100));

        // AXP192からのボタン状態をポーリングするか、割り込みをクリアすることもできます。
        // この部分も axp192 v0.2.0 の API に合わせて調整が必要かもしれません。
        // get_all_irqs() の戻り値やメソッド名が変わっている可能性があります。
        // ドキュメント: [https://docs.rs/axp192/0.2.0/axp192/](https://docs.rs/axp192/0.2.0/axp192/)
        if let Ok(irq_status) = axp.get_all_irqs() { // get_all_irqs が Result を返すか確認
            if irq_status.is_pek_short_press() { // is_pek_short_press が存在するか確認
                info!("AXP192 PEK Short Press detected!");
                axp.clear_pek_irq()?; // clear_pek_irq が存在するか確認
            }
        }
    }
}