use esp_idf_hal::{
    delay::FreeRtos,
    gpio::PinDriver,
    prelude::*,
    spi::{config::{Config as SpiConfig, DriverConfig}, SpiDeviceDriver, SpiDriver},
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::{Builder, interface::SpiInterface, models::ST7789};
use anyhow::Result;

static mut DISPLAY_BUFFER: [u8; 256] = [0u8; 256];

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    let sclk = peripherals.pins.gpio18;
    let mosi = peripherals.pins.gpio19;
    let cs = peripherals.pins.gpio5.into();

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        sclk,
        mosi,
        None,
        &DriverConfig::new(),
    )?;

    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        Some(cs),
        &SpiConfig::new().baudrate(10_000_000u32.Hz().into()),
    )?;

    let dc = PinDriver::output(peripherals.pins.gpio27)?;
    let mut bl = PinDriver::output(peripherals.pins.gpio15)?;
    bl.set_high()?; // バックライトON

    let mut delay = FreeRtos;

    let di = unsafe {
        SpiInterface::new(spi_device, dc, &mut DISPLAY_BUFFER)
    };

    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        //.invert_colors(ColorInversion::Inverted) // 外す
        .init(&mut delay)?;

    display.clear(Rgb565::BLACK)?;

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello TWatch 2020 V3!", Point::new(10, 120), style)
        .draw(&mut display)?;

    println!("Display initialized and text drawn!");

    // ボタン監視などなしで無限ループ（省略も可）
    loop {
        FreeRtos::delay_ms(1000);
    }
}
