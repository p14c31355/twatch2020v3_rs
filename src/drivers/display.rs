// src/drivers/display.rs
use anyhow::Result;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Gpio27, Gpio33, Output, PinDriver, AnyIOPin},
    spi::{SPI2, SpiDriver, SpiDeviceDriver, config::{Config as SpiConfig, DriverConfig as SpiDriverConfig}},
    units::FromValueType, // Add this line
};
use mipidsi::{Builder, Display, models::ST7789, interface::SpiInterface, options::ColorOrder};

pub struct TwatchDisplay<'a> {
    pub display: Display<
        SpiInterface<'a, SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Gpio27, Output>>,
        ST7789,
        PinDriver<'a, Gpio33, Output>,
    >,
    pub buffer: &'a mut [u8],
}

impl<'a> TwatchDisplay<'a> {
    pub fn new(
        spi2: SPI2,
        gpio18: esp_idf_hal::gpio::Gpio18,
        gpio23: esp_idf_hal::gpio::Gpio23,
        gpio5: esp_idf_hal::gpio::Gpio5,
        gpio27: Gpio27,
        gpio33: Gpio33,
        buffer: &'a mut [u8],
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

        let spi = SpiInterface::new(spi_device, dc, buffer);

        let display = Builder::new(ST7789, spi)
            .color_order(ColorOrder::Rgb)
            .reset_pin(rst)
            .init(&mut FreeRtos)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(Self {
            display,
            buffer,
        })
    }
}
