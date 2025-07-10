use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig},
    spi::{Spi, SpiConfig, SPI1, SPI2, SPI3},
    gpio::{PinDriver, AnyInputPin}, // Removed AnyOutputPin as it's not directly used for PinDriver output
    peripherals::Peripherals,
    prelude::{FromValueType, OutputPin},
    delay::FreeRtos,
    spi::{SpiDeviceDriver, SpiDriver},
};
use esp_idf_svc::sys::TickType_t;

// ディスプレイ関連のインポート
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorOrder, Orientation};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    text::Text,
};
use mipidsi::{Builder, models::ST7789};

use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// AXP192のI2Cアドレス（前回までと同じ）
const AXP192_ADDR: u8 = 0x34;
const AXP192_PEK_IRQ_EN1: u8 = 0x46;
const AXP192_PEK_IRQ_STATUS1: u8 = 0x48;
const AXP192_PEK_SHORT_PRESS_BIT: u8 = 0b0000_0010;

// Define an enum to hold different SPI peripherals
// This enum is not used in the corrected code, could be removed if not needed elsewhere.
enum AnySpi {
    Spi1(SPI1),
    Spi2(SPI2),
    Spi3(SPI3),
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Application started.");

    let peripherals = Peripherals::take().unwrap();
    let _delay = FreeRtos;

    // --- I2C AXP192 の初期化 ---
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    info!("Initializing I2C driver...");
    let mut i2c = I2cDriver::new(
        peripherals.i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()),
    )?;
    info!("I2C driver initialized successfully.");

    let i2c_timeout_ticks: TickType_t = 100u32;

    info!("Configuring AXP192 IRQ enable...");
    match i2c.write(
        AXP192_ADDR,
        &[AXP192_PEK_IRQ_EN1, AXP192_PEK_SHORT_PRESS_BIT],
        i2c_timeout_ticks,
    ) {
        Ok(_) => {
            info!("AXP192 configured for PEK IRQ!");
        },
        Err(e) => {
            error!("Failed to configure AXP192 IRQ enable: {:?}", e);
        }
    }

    info!("Clearing AXP192 IRQ status...");
    match i2c.write(
        AXP192_ADDR,
        &[AXP192_PEK_IRQ_STATUS1, 0xFF],
        i2c_timeout_ticks,
    ) {
        Ok(_) => {
            info!("AXP192 IRQ status cleared!");
        },
        Err(e) => {
            error!("Failed to clear AXP192 IRQ status: {:?}", e);
        }
    }

    // --- SPI ディスプレイの初期化 ---
    info!("Initializing SPI for display...");
    let sclk = peripherals.pins.gpio18;
    let sdo = peripherals.pins.gpio23; // MOSI
    let sdi_opt = Option::<AnyInputPin>::None; // MISO は未使用
    let cs = peripherals.pins.gpio5; // CS ピンを定義
    let dc = PinDriver::output(peripherals.pins.gpio27)?;
    let mut backlight = PinDriver::output(peripherals.pins.gpio4)?;

    // Spi::new takes the peripheral, SCLK, MOSI, MISO, CS, and SpiConfig.
    let spi_peripheral = Spi::new(
        peripherals.spi3, // SPI peripheral instance
        sclk,             // SCLK pin
        sdo,              // MOSI pin
        sdi_opt,          // MISO pin (None if not used)
        cs,               // CS pin
        &SpiConfig::new().baudrate(80.MHz().into()), // SpiConfig を使用
    )?;

    // SpiDeviceDriver::new now takes the Spi instance (which implements SpiDriver),
    // an optional CS pin (None if already handled by Spi), and the SpiConfig.
    let spi_device_driver = SpiDeviceDriver::new(
        spi_peripheral, // The Spi instance, which implements SpiDriver, not AnyInputPin
        Option::<AnyInputPin>::None, // CS pin is already handled by spi_peripheral, so None here
        &SpiConfig::new().baudrate(80.MHz().into()), // Re-use the config or define a new one for the device
    )?;
    
    info!("SPI driver initialized successfully.");

    // バックライトON
    backlight.set_high()?;
    info!("Display backlight ON.");

    // mipidsi の SpiInterface は spi_device_driver と dc を引数に取る
    let mut display_buffer = [0u8; 240 * 240 * 2]; // Buffer for mipidsi.
    let di = SpiInterface::new::<SpiDriver>(
        spi_device_driver.device(None, None)?, // No specific CS here, assuming it's handled by Spi::new
        dc.into_output().map_err(|e| anyhow::anyhow!("Failed to convert PinDriver to OutputPin: {:?}", e))?, // Pass the PinDriver<Output> directly
        &mut display_buffer
    );
    
    // ST7789V ディスプレイを初期化
    info!("Initializing ST7789V display controller...");
    let mut display = Builder::new(ST7789, di)
        .with_display_size(240, 240) // T-Watch 2020 V3 のディスプレイサイズ
        .with_orientation(Orientation::new()) // Use Orientation::new() as per mipidsi 0.9.0
        .with_invert_colors(ColorOrder::Rgb)
        .with_framebuffer_size(240, 240)
        .init(&mut _delay, None)?;
    info!("Display controller initialized.");

    // ディスプレイを黒でクリア
    info!("Clearing display with black color...");
    display.clear(Rgb565::BLACK).map_err(|e| anyhow::anyhow!("Display clear error: {:?}", e))?;
    info!("Display cleared.");

    // テキストを描画
    info!("Drawing text on display...");
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    Text::new("T-Watch 2020 V3 Test", Point::new(10, 50), text_style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Text draw error: {:?}", e))?;

    Text::new("I2C Status: FAILED", Point::new(10, 70), text_style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Text draw error: {:?}", e))?;

    info!("Text drawn. Display test successful if text is visible.");

    // --- ESP32 GPIO 35 (User Button) Initialization ---
    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();
    info!("Initializing GPIO35 for button...");
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;
    info!("GPIO35 pull-up/down implicitly handled (or not set).");
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    info!("GPIO35 interrupt type set.");
    unsafe { button.subscribe(move || { // Closure should take 0 arguments
        let mut pressed = button_pressed_clone.lock().unwrap();
        *pressed = true;
        info!("GPIO35 interrupt triggered!");
    }) }?;
    info!("GPIO35 subscribed to interrupts.");

    info!("Entering main loop...");

    loop {
        {
            let mut pressed = button_pressed.lock().unwrap();
            if *pressed {
                info!("Button Pressed (from ESP32 GPIO)!");
                *pressed = false;

                // AXP192の割り込みステータスを読み取り、クリアする
                let mut irq_status_buf = [0u8; 1];
                match i2c.write_read(
                    AXP192_ADDR,
                    &[AXP192_PEK_IRQ_STATUS1],
                    &mut irq_status_buf,
                    i2c_timeout_ticks,
                ) {
                    Ok(_) => {
                        let irq_status = irq_status_buf[0];
                        if (irq_status & AXP192_PEK_SHORT_PRESS_BIT) != 0 {
                            info!("AXP192 PEK Short Press detected (from I2C poll)!");
                        }
                        match i2c.write(
                            AXP192_ADDR,
                            &[AXP192_PEK_IRQ_STATUS1, irq_status],
                            i2c_timeout_ticks,
                        ) {
                            Ok(_) => {
                                info!("AXP192 IRQ status cleared via I2C.");
                            }
                            Err(e) => {
                                error!("Failed to clear AXP192 IRQ status: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read AXP192 IRQ status: {:?}", e);
                    }
                }

                thread::sleep(Duration::from_millis(50));
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}