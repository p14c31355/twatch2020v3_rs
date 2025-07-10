use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig}, // I2Cのインポートを追加
    gpio::{PinDriver, Pull},
    peripherals::Peripherals,
};
use esp_idf_svc::eventloop::*;
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// axp192 と embedded_hal のトレイトを追加
use axp192::{Axp192, ChargerState}; // ChargerState は必要に応じて
use embedded_hal::blocking::delay::DelayMs; // 遅延が必要な場合
use esp_idf_svc::hal::delay::FreeRtos; // 遅延プロバイダの例

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mut delay = FreeRtos; // AXP192の初期化用遅延

    // --- AXP192 (I2C) の初期化 ---
    // T-Watch 2020 V3 の I2C は通常 GPIO21 (SDA) と GPIO22 (SCL) です。
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let i2c = I2cDriver::new(
        peripherals.i2c0, // I2C0 ペリフェラルを使用
        sda,
        scl,
        &I2cConfig::new().baudrate(400.kHz().into()), // 高速I2C
    )?;

    // AXP192 ドライバーインスタンスを作成
    let mut axp = Axp192::new(i2c, delay)?;
    info!("AXP192 initialized!");

    // --- AXP192 固有の設定 (T-Watchのボタンには非常に重要) ---
    // これらは T-Watch のボタンと AXP192 を介した一般的な設定です。
    // ボタン割り込みや電源供給に使用されるGPIOを有効にします。
    // T-WatchのボタンはAXP192の電源キー(PEK)割り込みに接続されていることが多いです。
    axp.enable_pek_irq()?; // PEK (Power Enable Key) 割り込みを有効化
    axp.clear_all_irqs()?; // AXP192上の保留中の割り込みをクリア

    // 必要に応じて、AXP192のGPIOを構成します。T-Watchの回路図を確認して、
    // ボタン入力に関わるAXP192のGPIOがどのように使われているか確認してください。
    // axp.set_gpio0_mode_input()?; // 例: GPIO0をPMICの入力として設定

    // --- ESP32 GPIO 35 (ユーザーボタン) の初期化 ---
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;
    // 前述の通り、GPIO35には内部プルアップ/ダウンがありません。
    // T-Watchのボタンが外部プルアップされていると仮定し、NegEdgeが正しいです。
    // もしAXP192の割り込みを使用する場合、ESP32のGPIO35は単なる割り込み入力になります。
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    unsafe { button.subscribe(move || {
        let mut pressed = button_pressed_clone.lock().unwrap();
        *pressed = true;
    }) }?;

    // ... (メインループの残りの部分) ...
    loop {
        {
            let mut pressed = button_pressed.lock().unwrap();
            if *pressed {
                info!("Button Pressed!");
                *pressed = false;
                thread::sleep(Duration::from_millis(50)); // チャタリング防止
            }
        }
        thread::sleep(Duration::from_millis(100));

        // AXP192からのボタン状態をポーリングするか、割り込みをクリアすることもできます。
        // 例 (未テスト、AXP192のIRQ設定に依存します):
        // if let Ok(irq_status) = axp.get_all_irqs() {
        //     if irq_status.is_pek_short_press() {
        //         info!("AXP192 PEK Short Press detected!");
        //         axp.clear_pek_irq()?;
        //     }
        // }
    }
}