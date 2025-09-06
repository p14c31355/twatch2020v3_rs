use anyhow::Result;
use esp_idf_hal::spi::{config::DriverConfig as SpiDriverConfig, config::Config as SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver};
use esp_idf_hal::peripherals::Peripherals; // This line is already present
use mipidsi::{Builder, models::ST7789, Display};
use mipidsi::interface::SpiInterface; // Add this import
use embedded_graphics::pixelcolor::Rgb565; // Add this import
use esp_idf_hal::delay::FreeRtos; // Add this import
use esp_idf_hal::units::FromValueType; // Add this import
use mipidsi::options::ColorOrder; // Add this import

pub type TwatchDisplay<'a> = Display<SpiInterface<SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<Output>>, ST7789, Rgb565>;

pub fn init_display<'a>(p: Peripherals, buffer: &'a mut [u8]) -> Result<TwatchDisplay<'a>> {
    let driver = SpiDriver::new(
        p.spi2,
        p.pins.gpio18.into(),  // SCLK
        p.pins.gpio23.into(),  // MOSI (sdo)
        Some(p.pins.gpio19.into()), // MISO (sdi)
        &SpiDriverConfig::new(),
    )?;

    let dc = PinDriver::output(p.pins.gpio27)?;
    let rst = PinDriver::output(p.pins.gpio33)?;
    let cs = PinDriver::output(p.pins.gpio5)?;

    let spi_device = SpiDeviceDriver::new(
        &driver,
        Some(cs.into()),
        &SpiConfig::new().baudrate(26.MHz().into()),
    )?;

    let spi = SpiInterface::new(spi_device, dc, buffer);

    let display = Builder::new(ST7789, spi)
        .color_order(ColorOrder::Rgb)
        .reset_pin(rst)
        .init(&mut FreeRtos)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    Ok(display)
}
