use esp_idf_hal::{
    delay::FreeRtos,
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
    Spi(esp_idf_hal::spi::SpiError),
    Gpio(esp_idf_hal::gpio::GpioError),
    Mipidsi(mipidsi::builder::InitError<mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, esp_idf_hal::gpio::GpioError>, core::convert::Infallible>),
    Draw(mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, esp_idf_hal::gpio::GpioError>),
}

impl From<esp_idf_hal::spi::SpiError> for Error {
    fn from(e: esp_idf_hal::spi::SpiError) -> Self {
        Error::Spi(e)
    }
}
impl From<esp_idf_hal::gpio::GpioError> for Error {
    fn from(e: esp_idf_hal::gpio::GpioError) -> Self {
        Error::Gpio(e)
    }
}
impl From<mipidsi::builder::InitError<mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, esp_idf_hal::gpio::GpioError>, core::convert::Infallible>> for Error {
    fn from(e: mipidsi::builder::InitError<mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, esp_idf_hal::gpio::GpioError>, core::convert::Infallible>) -> Self {
        Error::Mipidsi(e)
    }
}
impl From<mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, esp_idf_hal::gpio::GpioError>> for Error {
    fn from(e: mipidsi::interface::SpiError<esp_idf_hal::spi::SpiError, esp_idf_hal::gpio::GpioError>) -> Self {
        Error::Draw(e)
    }
}

fn main() -> Result<(), Error> {
    esp_idf_svc::sys::link_patches();

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    // SPIピン設定
    let sclk = peripherals.pins.gpio18;
    let mosi = peripherals.pins.gpio19;
    let miso = None; // TFT_MISO NULL
    let cs = peripherals.pins.gpio5.into();
    let dc = PinDriver::output(peripherals.pins.gpio27)?;
    let mut bl = PinDriver::output(peripherals.pins.gpio15)?;

    // バックライトON
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

    // バッファは十分大きく(240*240/8=7200バイトより多めに)
    let mut display_buffer = [0u8; 4096];
    let di = SpiInterface::new(spi_device, dc, &mut display_buffer);

    // RSTピンなしの設定で初期化
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .reset_pin(None)  // RSTピンは接続していないためNone
        .rotation(mipidsi::Rotation::Rotate0)
        .init(&mut delay)?;

    delay.delay_ms(100); // 初期化後の余裕

    display.clear(Rgb565::BLACK)?;

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello TWatch 2020 V3!", Point::new(10, 120), style)
        .draw(&mut display)?;

    println!("Display initialized and text drawn!");

    loop {
        delay.delay_ms(1000);
    }
}
