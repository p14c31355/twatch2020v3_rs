use esp_idf_hal::gpio::AnyIOPin;

use esp_idf_hal::{
    delay::FreeRtos, gpio::{AnyOutputPin, PinDriver}, prelude::*, spi::{config::{Config as SpiConfig, DriverConfig}, SpiDeviceDriver, SpiDriver}
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};

use mipidsi::{models::ST7789, Builder};
use mipidsi::interface::SpiInterface;
use mipidsi::options::ColorInversion;
use anyhow::Result;

// embedded-hal v0.2互換用 ダミーDCピン定義
use embedded_hal::digital::OutputPin;
use core::convert::Infallible;

struct DummyNoopPin;

impl embedded_hal::digital::ErrorType for DummyNoopPin {
    type Error = Infallible;
}

impl OutputPin for DummyNoopPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// SPIインタフェース用バッファ
static mut DISPLAY_BUFFER: [u8; 256] = [0u8; 256];

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();

    let sclk = peripherals.pins.gpio18;
    let sdo  = peripherals.pins.gpio19;
    let sdi: Option<AnyIOPin> = None;
    let cs: AnyOutputPin = peripherals.pins.gpio5.into();

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        sclk,
        sdo,
        sdi,
        &DriverConfig::new(),
    )?;

    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        Some(cs),
        &SpiConfig::new().baudrate(20.MHz().into()),
    )?;

    let dc = PinDriver::output(peripherals.pins.gpio27)?;
    let mut bl = PinDriver::output(peripherals.pins.gpio15)?;
    bl.set_high()?; // バックライトON

    let mut display_buffer = [0u8; 256];
    let di = SpiInterface::new(spi_device, dc, &mut display_buffer);



    let mut delay = FreeRtos;

    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello TWatch 2020 V3!", Point::new(20, 120), text_style)
        .draw(&mut display)
        .unwrap();

    println!("Display initialized!");

    let button = PinDriver::input(peripherals.pins.gpio0)?;

    loop {
        if button.is_low() {
            println!("Button pressed!");
        }
        FreeRtos::delay_ms(100);
    }
}
