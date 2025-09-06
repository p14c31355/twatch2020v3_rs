use anyhow::Result;
use esp_idf_hal::spi::{config::DriverConfig as SpiDriverConfig, config::Config as SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver, Gpio27, Gpio33};
use esp_idf_hal::peripherals::Peripherals;
use mipidsi::{Builder, models::ST7789, Display};
use mipidsi::interface::SpiInterface;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::units::FromValueType;
use mipidsi::options::ColorOrder;

// The SpiDeviceDriver now owns the SpiDriver.
pub type TwatchDisplay<'a> = Display<SpiInterface<'a, SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Gpio27, Output>>, ST7789, PinDriver<'a, Gpio33, Output>>;

pub fn init_display<'a>(
    spi2: esp_idf_hal::spi::SPI2,
    gpio18: esp_idf_hal::gpio::Gpio18,
    gpio23: esp_idf_hal::gpio::Gpio23,
    gpio5: esp_idf_hal::gpio::Gpio5,
    gpio27: esp_idf_hal::gpio::Gpio27,
    gpio33: esp_idf_hal::gpio::Gpio33,
    buffer: &'a mut [u8],
) -> Result<TwatchDisplay<'a>> {
    // Create the SpiDriver on the stack.
    let driver = SpiDriver::new(
        spi2,
        gpio18,
        gpio23,
        None::<AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;

    // Move the driver into the SpiDeviceDriver.
    let spi_device = SpiDeviceDriver::new(
        driver,
        Some(gpio5),
        &SpiConfig::new().baudrate(26.MHz().into()),
    )?;

    let dc: PinDriver<'_, Gpio27, Output> = PinDriver::output(gpio27)?;
    let rst: PinDriver<'_, Gpio33, Output> = PinDriver::output(gpio33)?;

    // Move the spi_device into the SpiInterface.
    let spi = SpiInterface::new(spi_device, dc, buffer);

    let display = Builder::new(ST7789, spi)
        .color_order(ColorOrder::Rgb)
        .reset_pin(rst)
        .init(&mut FreeRtos)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    Ok(display)
}
