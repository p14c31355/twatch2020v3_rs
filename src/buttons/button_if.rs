use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_sys::EspError;
use esp_idf_svc::log::*; // log マクロを使う為に必要
use log::*; // log levelを使う為に必要
use std::time::Duration;
use std::thread;

fn main() -> Result<(), EspError> {
    EspLogger::initialize_default(); // ロギングの初期化

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;

    let mut last_state = pin.is_high(); // 初期状態を保存
    let debounce_delay = Duration::from_millis(50); // チャタリング対策の遅延

    loop {
        let current_state = pin.is_high(); // 現在の状態を取得

        if current_state != last_state {
            // 状態が変化した場合、チャタリング対策の遅延
            thread::sleep(debounce_delay);
            if pin.is_low() == current_state {
                // 遅延後も状態が変わらなければ、有効な状態変化とみなす
                if current_state {
                    info!("HelloButton!"); // ログメッセージを出力
                }
                last_state = current_state; // 状態を更新
            }
        }
        thread::sleep(Duration::from_millis(10)); // 状態確認の間隔
    }
}