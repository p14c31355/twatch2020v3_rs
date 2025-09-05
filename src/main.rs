use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, AnyOutputPin, PinDriver}, // AnyIOPin を追加
    peripherals::Peripherals,
    prelude::*,
    spi::{config::{Config as SpiConfig, DriverConfig}, SpiDeviceDriver, SpiDriver},
    sys::EspError,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::{Builder, interface::SpiInterface, models::ST7789};

use std::{thread, time::Duration};
use esp_idf_hal::{
    prelude::*,
    ledc::{config::TimerConfig as LedcTimerConfig, LedcDriver, LedcTimerDriver, Resolution},
    timer::Timer,
};
use anyhow::Result;

#[derive(Debug)]
enum Error {
    Esp(EspError),
    Gpio(esp_idf_hal::gpio::GpioError),
    Spi(esp_idf_hal::spi::SpiError),
    MipidsiInit(String),
    Draw(String),
}

impl From<EspError> for Error {
    fn from(e: EspError) -> Self { Error::Esp(e) }
}
impl From<esp_idf_hal::gpio::GpioError> for Error {
    fn from(e: esp_idf_hal::gpio::GpioError) -> Self { Error::Gpio(e) }
}
impl From<esp_idf_hal::spi::SpiError> for Error {
    fn from(e: esp_idf_hal::spi::SpiError) -> Self { Error::Spi(e) }
}

// 適切なバッファサイズを調整してください。240x240ピクセル、Rgb565（2バイト/ピクセル）なら
// 240 * 240 * 2 = 115200バイトが必要です。
// DISPLAY_BUFFERのサイズが足りないと、描画エラーやクラッシュの原因になります。
// `mipidsi`の `SpiInterface::new` は `WriteOnlyDataCommand` を使う場合、
// バッファサイズは転送するデータの最大サイズに合わせる必要があります。
// 全画面描画をバッファで行うなら 115200 を確保すべきですが、
// 通常は描画コマンドの引数や部分描画のために使うので、転送効率が良い範囲で小さくします。
// ここでは、例として一般的な転送バッファサイズとして少し大きめに設定します。
// 実際にフルスクリーン更新を行うにはもっと大きいか、部分的な描画を繰り返すロジックが必要です。
// ここはデモ目的で、一旦大きめに設定しますが、実際にメモリに乗り切らない可能性もあります。
// プロジェクトの必要に応じて調整してください。
static mut DISPLAY_BUFFER: [u8; 16384] = [0u8; 16384]; // 240x240 LCD用ならもっと大きいか、DMAを使うべき。

fn main() -> Result<(), Error> {
    esp_idf_sys::link_patches();
    let peripherals = Peripherals::take().unwrap();

    let timer = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &LedcTimerConfig::new().resolution(Resolution::Bits8).frequency(100.Hz()),
    )?;
    let mut backlight = LedcDriver::new(
        peripherals.ledc.channel0,
        &timer,
        peripherals.pins.gpio15,
    )?;

    println!("バックライト: 50%");
    backlight.set_duty(128)?;
    thread::sleep(Duration::from_secs(2));

    println!("バックライト: Max 100%");
    backlight.set_duty(255)?;
    thread::sleep(Duration::from_secs(2));

    println!("バックライト: 0%");
    backlight.set_duty(0)?;
    thread::sleep(Duration::from_secs(2));

    Ok(())
}
