// src/drivers/display.rs
use anyhow::Result;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Gpio27, Gpio33, Output, PinDriver, AnyIOPin},
    spi::{SPI2, SpiDriver, SpiDeviceDriver, config::{Config as SpiConfig, DriverConfig as SpiDriverConfig}},
    units::FromValueType,
};
use mipidsi::{Builder, Display, models::ST7789, interface::SpiInterface, options::ColorOrder};
use embedded_graphics::{geometry::Size, prelude::RgbColor};
use embedded_graphics::pixelcolor::Rgb565;
use display_interface_spi::SPIInterfaceNoCS;

pub struct TwatchDisplay<'a> {
    pub display: Display<
        SpiInterface<'a, SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Gpio27, Output>>,
        ST7789,
        PinDriver<'a, Gpio33, Output>,
    >,
}

impl<'a> TwatchDisplay<'a> {
    pub fn new(
        spi2: SPI2,
        gpio18: esp_idf_hal::gpio::Gpio18,
        gpio23: esp_idf_hal::gpio::Gpio23,
        gpio5: esp_idf_hal::gpio::Gpio5,
        gpio27: Gpio27,
        gpio33: Gpio33,
    ) -> Result<Self> {
        let driver = SpiDriver::new(
            spi2,
            gpio18,
            gpio23,
            None::<AnyIOPin>,
            &SpiDriverConfig::new(),
        )?;

        let spi_device = SpiDeviceDriver::new(
            driver,
            Some(gpio5),
            &SpiConfig::new().baudrate(26.MHz().into()),
        )?;

        let dc: PinDriver<Gpio27, Output> = PinDriver::output(gpio27)?;
        let rst: PinDriver<Gpio33, Output> = PinDriver::output(gpio33)?;

        let spi = SPIInterfaceNoCS::new(spi_device, dc);

        let display = Builder::new(ST7789, spi)
            .with_display_size(240, 240)
            .with_color_order(ColorOrder::Rgb)
            .with_reset_pin(rst)
            .init(&mut FreeRtos, Rgb565::BLACK)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(Self {
            display,
        })
    }
}