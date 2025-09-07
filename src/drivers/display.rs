// src/drivers/display.rs
use anyhow::Result;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Gpio27, Gpio33, Output, PinDriver},
    spi::{
        SPI2, SpiDeviceDriver, SpiDriver,
        config::{Config as SpiConfig, DriverConfig as SpiDriverConfig},
    },
    units::FromValueType,
};
use mipidsi::{Builder, Display, interface::SpiInterface, models::ST7789, options::ColorOrder};

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
        gpio19: esp_idf_hal::gpio::Gpio19,
        gpio5: esp_idf_hal::gpio::Gpio5,
        gpio27: Gpio27,
        gpio33: Gpio33,
        display_buffer: &'a mut [u8],
    ) -> Result<Self> {
        let driver = SpiDriver::new(
            spi2,
            gpio18,
            gpio19,
            None::<esp_idf_hal::gpio::AnyIOPin>,
            &SpiDriverConfig::new(),
        )?;

        let spi_device = SpiDeviceDriver::new(
            driver,
            Some(gpio5),
            &SpiConfig::new().baudrate(10.MHz().into()),
        )?;

        let dc: PinDriver<Gpio27, Output> = PinDriver::output(gpio27)?;
        let rst: PinDriver<Gpio33, Output> = PinDriver::output(gpio33)?;

        let spi = SpiInterface::new(spi_device, dc, display_buffer);

        let display = Builder::new(ST7789, spi)
            .display_size(240, 240)
            .color_order(ColorOrder::Rgb)
            .reset_pin(rst)
            .display_offset(0, 0)
            .init(&mut FreeRtos)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(Self { display })
    }

    pub fn draw_rectangle(&mut self, rect: Rectangle) -> Result<()> {
        rect.into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }

    pub fn safe_draw<D>(&mut self, drawable: &D) -> Result<()>
    where
        D: embedded_graphics::Drawable<Color = Rgb565>,
    {
        FreeRtos::delay_ms(1);
        drawable.draw(&mut self.display).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }

    pub fn safe_fill_rect(&mut self, rect: Rectangle, color: Rgb565) -> Result<()> {
        FreeRtos::delay_ms(1);
        rect.into_styled(PrimitiveStyle::with_fill(color))
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }
}
