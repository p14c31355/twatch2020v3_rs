use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyInputPin, AnyOutputPin, Gpio23, Gpio18, Gpio19, Gpio5, PinDriver},
    prelude::*,
    spi::{config::Config as SpiConfig, Driver as SpiDriver, DeviceDriver as SpiDeviceDriver, DriverConfig, SPI2},
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::Builder;
use st7789::ST7789;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    // Peripherals取得
    let peripherals = Peripherals::take().unwrap();

    // SPIピンの設定
    let sclk = peripherals.pins.gpio18;
    let sdo  = peripherals.pins.gpio23;
    let sdi  = peripherals.pins.gpio19;
    let cs   = peripherals.pins.gpio5;

    // SPIドライバの初期化
    let spi_peripheral = SpiDriver::new(
        peripherals.spi2,
        sclk,
        sdo,
        sdi,
        &DriverConfig::new(),
    )?;

    // デバイスドライバの初期化（CSピンをNoneに）
    let spi_device_driver = SpiDeviceDriver::new(
        spi_peripheral,
        Option::<AnyOutputPin>::None, // CSピンは未使用
        &SpiConfig::new().baudrate(80.MHz().into()),
    )?;

    // Displayインタフェース作成
    let di = mipidsi::DisplayInterface::new_no_cs(spi_device_driver);

    // ST7789用ディスプレイ初期化
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .build()
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello TWatch 2020 V3!", Point::new(20, 120), text_style)
        .draw(&mut display)
        .unwrap();

    println!("Display initialized!");

    // ボタンの例 (仮)
    let button = PinDriver::input(peripherals.pins.gpio0)?;

    unsafe {
        button.subscribe(move || {
            println!("Button pressed!");
        });
    }

    FreeRtos::delay_ms(1000);

    Ok(())
}
