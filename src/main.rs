use esp_idf_hal::{
    delay::FreeRtos,
    sys::EspError,
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

#[derive(Debug)]
enum Error {
    Esp(EspError),
    Gpio(esp_idf_hal::gpio::GpioError),
    Spi(esp_idf_hal::spi::SpiError),
    Mipidsi(Box<dyn std::error::Error + Send + Sync>),
    Draw(embedded_graphics::prelude::DrawingError),
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
impl From<embedded_graphics::prelude::DrawingError> for Error {
    fn from(e: embedded_graphics::prelude::DrawingError) -> Self { Error::Draw(e) }
}

fn main() -> Result<(), Error> {
    esp_idf_svc::sys::link_patches();

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    let sclk = peripherals.pins.gpio18;
    let mosi = peripherals.pins.gpio19;
    let miso = None;
    let cs = peripherals.pins.gpio5.into();

    let dc = PinDriver::output(peripherals.pins.gpio27)?;
    let mut bl = PinDriver::output(peripherals.pins.gpio15)?;

    bl.set_high()?;

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        sclk,
        mosi,
        miso,
        &DriverConfig::new(),
    )?;

    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        Some(cs),
        &SpiConfig::new().baudrate(10_000_000u32.Hz().into()),
    )?;

    let mut delay = FreeRtos;

    let mut display_buffer = [0u8; 4096];
    let di = SpiInterface::new(spi_device, dc, &mut display_buffer);

    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        // .reset_pin(...)  // RSTピンなしなら呼ばない
        .init(&mut delay)
        .map_err(|e| Error::Mipidsi(Box::new(e)))?;

    FreeRtos::delay_ms(100);

    display.clear(Rgb565::BLACK)?;

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello TWatch 2020 V3!", Point::new(10, 120), style)
        .draw(&mut display)?;

    println!("Display initialized and text drawn!");

    loop {
        FreeRtos::delay_ms(1000);
    }
}
