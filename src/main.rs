use esp_idf_svc::hal::{
    gpio::Pull,
    peripherals::Peripherals,
};
use esp_idf_svc::eventloop::*;
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    // ここを gpio21 から gpio35 に変更
    let mut button = esp_idf_svc::hal::gpio::PinDriver::input(peripherals.pins.gpio35)?;
    // IO35は内部プルアップがないため、外部プルアップが必要
    // もしボタンとGNDの間に外部プルアップ抵抗(10kΩなど)があるならOK。
    // なければ、通常はPull::Upを設定しない、またはPull::Downに設定して
    // ボタンが押されたときにVCCに接続されるように配線を変更する必要がある。
    // 今回のT-Watchのボタンは通常内部にプルアップ/ダウンがないので、
    // 物理的な接続に依存します。
    // ただし、多くの開発ボードのオンボードボタンは、デフォルトでプルアップ/ダウンされています。
    // T-Watch 2020 V3のUser Buttonは、外部にプルアップされているケースが多いです。
    // その場合、Pull::Upは不要です。
    // ここで Pull::Up を削除するか、コメントアウトしてみてください。
    // button.set_pull(Pull::Up)?; // IO35 は内部プルアップがないため削除またはコメントアウト

    // 割り込みハンドラの初期化
    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();

    // 割り込み設定
    // 外部プルアップがあるなら NegEdge (押したらLOW)
    // 外部プルダウンがあるなら PosEdge (押したらHIGH)
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
                // チャタリング防止のために少し待つと良いかもしれません
                thread::sleep(Duration::from_millis(50));
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}