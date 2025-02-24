use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_sys::*;
use log::info;
use tokio;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mut pin = PinDriver::input(peripherals.pins.gpio35)?; // GPIO35を入力モードに設定

    tokio::spawn(async move {
        let mut last_state = pin.is_low(); // 初期状態を保存
        let debounce_delay = Duration::from_millis(50); // チャタリング対策の遅延

        loop {
            let current_state = pin.is_low(); // 現在の状態を取得

            if current_state != last_state {
                // 状態が変化した場合、チャタリング対策の遅延
                thread::sleep(debounce_delay);
                if pin.is_low() == current_state {
                    // 遅延後も状態が変わらなければ、有効な状態変化とみなす
                    if current_state {
                        info!("HelloButton!");
                    } else {
                        info!("Button released!");
                    }
                    last_state = current_state; // 状態を更新
                }
            }
            thread::sleep(Duration::from_millis(10)); // 状態確認の間隔
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}