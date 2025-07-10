use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig}, // I2Cはそのまま残します
    spi::{
        SpiDriver, SpiConfig, SpiMode,
        Master,
    },
    gpio::{PinDriver, Output, Input, Pin}, // Inputも必要なので追加
    peripherals::Peripherals,
    prelude::FromValueType,
    delay::FreeRtos,
};
use esp_idf_svc::sys::TickType_t;

// ディスプレイ関連のインポート
use display_interface_spi::SpiInterface;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    text::Text,
};
use mipidsi::Builder;

use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// AXP192のI2Cアドレス（前回までと同じ）
const AXP192_ADDR: u8 = 0x34;
const AXP192_PEK_IRQ_EN1: u8 = 0x46;
const AXP192_PEK_IRQ_STATUS1: u8 = 0x48;
const AXP192_PEK_SHORT_PRESS_BIT: u8 = 0b0000_0010;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Application started.");

    let peripherals = Peripherals::take().unwrap();
    let _delay = FreeRtos;

    // --- I2C AXP192 の初期化 (ボタン検出のため、前回と同じコードを残します) ---
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
            // ここでreturn Err(e.into()); をコメントアウトし、
            // ディスプレイテストを続行できるようにします。
            // return Err(e.into());
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
            // return Err(e.into());
        }
    }

    // --- SPI ディスプレイの初期化 ---
    info!("Initializing SPI for display...");
    let sclk = peripherals.pins.gpio18;
    let sdo = peripherals.pins.gpio23; // MOSI
    // MISO (SDI) は使わないので None
    let spi_driver = SpiDriver::new(
        peripherals.spi2, // 通常、ESP32-WROVERはSPI2またはSPI3を使用
        sclk,
        sdo,
        Option::<PinDriver<'_, Input>>::None, // MISOをNoneで渡す
        &SpiConfig::new().baudrate(80.MHz().into()), // 高速SPI通信
    )?;
    info!("SPI driver initialized successfully.");

    let dc = PinDriver::output(peripherals.pins.gpio27)?;
    let rst = PinDriver::output(peripherals.pins.gpio34)?;
    let cs = PinDriver::output(peripherals.pins.gpio5)?;
    let mut backlight = PinDriver::output(peripherals.pins.gpio4)?; // バックライト

    // バックライトON
    backlight.set_high()?;
    info!("Display backlight ON.");

    let di = SpiInterface::new(spi_driver, dc, cs);

    // ST7789V ディスプレイを初期化
    info!("Initializing ST7789V display controller...");
    let mut display = Builder::st7789(di)
        .with_display_size(240, 240) // T-Watch 2020 V3の解像度
        .with_invert_colors(mipidsi::ColorOrder::Rgb) // 必要に応じて調整
        .with_framebuffer_size(240, 240)
        .init(&mut _delay, Some(rst)) // _delay は FreeRtos のインスタンス
        .map_err(|e| anyhow::anyhow!("Display init error: {:?}", e))?; // エラーハンドリング
    info!("Display controller initialized.");

    // ディスプレイを黒でクリア
    info!("Clearing display with black color...");
    display.clear(Rgb565::BLACK).map_err(|e| anyhow::anyhow!("Display clear error: {:?}", e))?;
    info!("Display cleared.");

    // テキストを描画
    info!("Drawing text on display...");
    let text_style = MonoTextStyle::new(FONT_6X10, Rgb565::WHITE);
    Text::new("T-Watch 2020 V3 Test", Point::new(10, 50), text_style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Text draw error: {:?}", e))?;

    Text::new("I2C Status: FAILED", Point::new(10, 70), text_style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Text draw error: {:?}", e))?;

    info!("Text drawn. Display test successful if text is visible.");


    // --- ESP32 GPIO 35 (User Button) Initialization (前回と同じコード) ---
    let button_pressed = Arc::new(Mutex::new(false)); // 再宣言ではなく、これを使う
    let button_pressed_clone = button_pressed.clone(); // 再宣言ではなく、これを使う
    info!("Initializing GPIO35 for button...");
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;
    info!("GPIO35 pull-up/down implicitly handled (or not set).");
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    info!("GPIO35 interrupt type set.");
    unsafe { button.subscribe(move || {
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