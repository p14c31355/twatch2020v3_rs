use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_sys::*;
use log::info;
use tokio;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use esp_idf_hal::gpio::InterruptPinDriver;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?; // GPIO35を入力モードに設定
    let mut interrupt_pin = InterruptPinDriver::new(peripherals.pins.gpio35)?; //割り込み入力設定

    let (tx, rx) = mpsc::channel();

    interrupt_pin.set_interrupt_type(InterruptType::NegEdge)?; // 割り込みタイプを設定(プルアップ配線の場合)
    interrupt_pin.enable_interrupt()?; // 割り込みを有効化

    interrupt_pin.subscribe(move || {
        tx.send(()).unwrap();
    })?;

    tokio::spawn(async move {
        loop {
            rx.recv().unwrap();
            info!("HelloButton!");
            thread::sleep(Duration::from_millis(200)); // チャタリング対策
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    Ok(())
}