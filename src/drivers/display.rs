use anyhow::Result;
use esp_idf_hal::spi::{config::DriverConfig as SpiDriverConfig, config::Config as SpiConfig, SpiDeviceDriver, SpiDriver};
use esp_idf_hal::gpio::{AnyIOPin, Output, PinDriver, Gpio27, Gpio33, Gpio19, Gpio18, Gpio23, Gpio5};
use esp_idf_hal::peripherals::Peripherals;
use mipidsi::{Builder, models::ST7789, Display};
use mipidsi::interface::SpiInterface;
use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::units::FromValueType;
use mipidsi::options::ColorOrder;

pub type TwatchDisplay<'a> = Display<SpiInterface<'a, SpiDeviceDriver<'a, &'a SpiDriver<'a>>, PinDriver<'a, Gpio27, Output>>, ST7789, PinDriver<'a, Gpio33, Output>>;

pub fn init_display<'a>(p: Peripherals, buffer: &'a mut [u8]) -> Result<TwatchDisplay<'a>> {
    let sclk_pin = PinDriver::output(p.pins.gpio18)?.into_any_io_pin();
    let mosi_pin = PinDriver::output(p.pins.gpio23)?.into_any_io_pin();
    let miso_pin = PinDriver::input(p.pins.gpio19)?.into_any_io_pin();

    let driver = SpiDriver::new(
        p.spi2,
        sclk_pin,
        mosi_pin,
        Some(miso_pin),
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
