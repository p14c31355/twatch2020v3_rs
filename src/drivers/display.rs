use anyhow::Result;
use esp_idf_hal::spi::{config::DriverConfig as SpiDriverConfig, config::Config as SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, Output, PinDriver, Gpio27, Gpio33};
use esp_idf_hal::peripherals::Peripherals; // This line is already present
use mipidsi::{Builder, models::ST7789, Display};
use mipidsi::interface::SpiInterface; // Add this import
use embedded_graphics::pixelcolor::Rgb565; // Add this import
use esp_idf_hal::delay::FreeRtos; // Add this import
use esp_idf_hal::units::FromValueType; // Add this import
use mipidsi::options::ColorOrder; // Add this import

pub type TwatchDisplay<'a> = Display<SpiInterface<'a, SpiDeviceDriver<'a, &'a SpiDriver<'a>>, PinDriver<'a, Gpio27, Output>>, ST7789, PinDriver<'a, Gpio33, Output>>;

pub fn init_display<'a>(p: Peripherals, buffer: &'a mut [u8]) -> Result<TwatchDisplay<'a>> {
    let sclk_pin = p.pins.gpio18.into().into_output().into_any_io_pin();
    let mosi_pin = p.pins.gpio23.into().into_output().into_any_io_pin();
    let miso_pin_driver: PinDriver<'_, Gpio19, esp_idf_hal::gpio::Input> = PinDriver::input(p.pins.gpio19)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let driver = SpiDriver::new(
        p.spi2,
        sclk_pin,  // SCLK
        mosi_pin,  // MOSI (sdo)
        Some(miso_pin_driver.into_any_io_pin()), // MISO (sdi)
        &SpiDriverConfig::new(),
    )?;

    let dc: PinDriver<'_, Gpio27, Output> = PinDriver::output(p.pins.gpio27)?;
    let rst: PinDriver<'_, Gpio33, Output> = PinDriver::output(p.pins.gpio33)?;
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
