use esp_idf_svc::hal::{
    gpio::Pull, // Gpio21, Input, Pin は不要になったため削除
    peripherals::Peripherals,
};
use esp_idf_svc::eventloop::*;
// use esp_idf_svc::sys::*; // 直接使用しないため削除
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // 初期化
    esp_idf_svc::sys::link_patches(); // これはesp_idf_svc::sys::*からの唯一の使用箇所
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    // 変更点: peripherals.pins.gpio21 を直接 PinDriver::input に渡します
    let mut button = esp_idf_svc::hal::gpio::PinDriver::input(peripherals.pins.gpio21)?;
    button.set_pull(Pull::Up)?;

    // イベントループの初期化
    let event_loop = EspSystemEventLoop::take()?; // この例では直接使われていませんが、文脈のために残しています。

    // 割り込みハンドラの初期化
    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();

    // 割り込み設定
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    unsafe { button.subscribe(move || {
        let mut pressed = button_pressed_clone.lock().unwrap();
        *pressed = true;
    }) }?;

    // メインループ
    loop {
        {
            let mut pressed = button_pressed.lock().unwrap();
            if *pressed {
                info!("Button Pressed!");
                *pressed = false;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}