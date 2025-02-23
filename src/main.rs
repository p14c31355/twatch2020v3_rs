use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_sys::EspError;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;

    loop {
        if pin.is_high() { // ボタンが押されたとき（LOWになる場合）
            println!("HelloButton!");
            // ボタンが離されるまで待機
            while pin.is_high() {
                thread::sleep(Duration::from_millis(10));
            }
        }
        thread::sleep(Duration::from_millis(10)); // 状態確認の間隔
    }
}