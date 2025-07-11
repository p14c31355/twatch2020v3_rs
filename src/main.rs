use std::fmt;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::AnyIOPin,
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

// 表示用のバッファをstatic mutでグローバルに確保
static mut DISPLAY_BUFFER: [u8; 4096] = [0; 4096];

// 自作のエラーenum
#[derive(Debug)]
enum MyError {
    MipidsiInitError(String),
    SpiError(String),
    GpioError(String),
    Other(anyhow::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::MipidsiInitError(e) => write!(f, "Mipidsi init error: {}", e),
            MyError::SpiError(e) => write!(f, "SPI error: {}", e),
            MyError::GpioError(e) => write!(f, "GPIO error: {}", e),
            MyError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for MyError {}

impl From<anyhow::Error> for MyError {
    fn from(e: anyhow::Error) -> Self {
        MyError::Other(e)
    }
}

fn main() -> Result<(), MyError> {
    esp_idf_svc::sys::link_patches();

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    let sclk = peripherals.pins.gpio18;
    let mosi = peripherals.pins.gpio19;
    let miso: Option<AnyIOPin> = None;
    let cs: esp_idf_hal::gpio::AnyOutputPin = peripherals.pins.gpio5.into();

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        sclk,
        mosi,
        miso,
        &DriverConfig::new(),
    ).map_err(|e| MyError::SpiError(format!("{:?}", e)))?;

    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        Some(cs),
        &SpiConfig::new().baudrate(10_000_000u32.Hz().into()),
    ).map_err(|e| MyError::SpiError(format!("{:?}", e)))?;

    let dc = PinDriver::output(peripherals.pins.gpio27)
        .map_err(|e| MyError::GpioError(format!("{:?}", e)))?;
    let mut bl = PinDriver::output(peripherals.pins.gpio15)
        .map_err(|e| MyError::GpioError(format!("{:?}", e)))?;
    bl.set_high().map_err(|e| MyError::GpioError(format!("{:?}", e)))?;

    let mut delay = FreeRtos;

    // unsafeブロックでstatic mutを参照
    let di = unsafe {
        SpiInterface::new(spi_device, dc, &raw mut DISPLAY_BUFFER)
    };

    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .init(&mut delay)
        .map_err(|e| MyError::MipidsiInitError(format!("{:?}", e)))?;

    display.clear(Rgb565::BLACK)
        .map_err(|e| MyError::SpiError(format!("{:?}", e)))?;

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello TWatch 2020 V3!", Point::new(10, 120), style)
        .draw(&mut display)
        .map_err(|e| MyError::SpiError(format!("{:?}", e)))?;

    println!("Display initialized and text drawn!");

    loop {
        FreeRtos::delay_ms(1000);
    }
}
