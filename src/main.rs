use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyOutputPin, PinDriver},
    prelude::*,
    spi::{config::Config as SpiConfig, config::DriverConfig, SpiDeviceDriver, SpiDriver},
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};

use mipidsi::{models::ST7789, Builder};
use mipidsi::interface::SpiInterface;
use anyhow::Result;

// embedded-hal v0.2互換用 ダミーDCピン定義
use embedded_hal::digital::v2::OutputPin;

struct DummyNoopPin;

impl OutputPin for DummyNoopPin {
    type Error = core::convert::Infallible;
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

// SPIインタフェース用バッファ
static mut DISPLAY_BUFFER: [u8; 256] = [0u8; 256];

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();

    // SPIピンの設定
    let sclk = peripherals.pins.gpio18;
    let sdo  = peripherals.pins.gpio23;
    let sdi  = Some(peripherals.pins.gpio19);

    // SPIドライバの初期化
    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        sclk,
        sdo,
        sdi,
        &DriverConfig::new(),
    )?;

    // SPIデバイスドライバ (CSピンなし)
    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        Option::<AnyOutputPin>::None,
        &SpiConfig::new().baudrate(80.MHz().into()),
    )?;

    // ディスプレイインタフェース作成 (DCピン無し)
    let di = unsafe {
        SpiInterface::new(spi_device, DummyNoopPin, &mut DISPLAY_BUFFER)
    };

    // ディスプレイ初期化
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .invert_colors(true)
        .init()
        .unwrap();

    // 画面を黒でクリア
    display.clear(Rgb565::BLACK).unwrap();

    // フォントスタイル設定
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    // テキスト描画
    Text::new("Hello TWatch 2020 V3!", Point::new(20, 120), text_style)
        .draw(&mut display)
        .unwrap();

    println!("Display initialized!");

    // GPIO0ボタンの入力ピンとしての設定
    let button = PinDriver::input(peripherals.pins.gpio0)?;

    // 簡易ポーリングループ
    loop {
        if button.is_low() {
            println!("Button pressed!");
        }
        FreeRtos::delay_ms(100);
    }
}
