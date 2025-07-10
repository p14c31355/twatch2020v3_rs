use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyOutputPin, PinDriver},
    prelude::*,
    spi::{config::Config as SpiConfig, config::DriverConfig, SpiDeviceDriver, SpiDriver, SPI2},
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};

use mipidsi::{models::ST7789, Builder, DisplayConfig, ModelOptions};
use anyhow::Result;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    // Peripherals取得
    let peripherals = Peripherals::take().unwrap();

    // SPIピンの設定
    let sclk = peripherals.pins.gpio18;
    let sdo  = peripherals.pins.gpio23;
    let sdi  = peripherals.pins.gpio19;

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

    // Displayインタフェース作成
    let di = mipidsi::DisplayInterface::new_no_cs(spi_device);

    // ModelOptionsの設定（必要なら細かく調整可）
    let options = ModelOptions {
        invert_colors: true, // T-Watch 2020 V3 は通常true
        ..Default::default()
    };

    // ST7789用ディスプレイ初期化
    let mut display = Builder::new(di, ST7789, options)
        .display_size(240, 240)
        .display_orientation(DisplayConfig::default())
        .init()
        .unwrap();

    // ディスプレイ描画
    display.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello TWatch 2020 V3!", Point::new(20, 120), text_style)
        .draw(&mut display)
        .unwrap();

    println!("Display initialized!");

    // ボタン入力例（GPIO0を入力に）
    let button = PinDriver::input(peripherals.pins.gpio0)?;

    // 擬似 subscribe 処理（簡易ポーリング）
    loop {
        if button.is_low() {
            println!("Button pressed!");
        }
        FreeRtos::delay_ms(100);
    }
}
