use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_sys::*;
use log::info;
use async_std;
use async_std::channel::{bounded, Sender};
use std::time::Duration;

async fn monitor_gpio(pin: PinDriver<'static, esp_idf_hal::gpio::Gpio35, esp_idf_hal::gpio::Input>, tx: Sender<bool>) {
    let mut last_state = pin.is_low();
    loop {
        let current_state = pin.is_low();
        if current_state != last_state {
            if tx.send(current_state).await.is_ok() {
                last_state = current_state;
                async_std::task::sleep(Duration::from_millis(1000)).await; // チャタリング対策
            }
        }
        async_std::task::sleep(Duration::from_millis(200)).await; // ポーリング間隔
    }
}

#[async_std::main]
async fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35).unwrap();

    let (tx, rx) = bounded(1); // バッファサイズ1のチャネルを作成

    async_std::task::spawn(monitor_gpio(pin, tx)); // GPIO監視タスクを生成

    loop {
        if let Ok(state) = rx.recv().await {
            if state {
                info!("Button pressed!");
            } else {
                info!("Button released!");
            }
        }
    }
}