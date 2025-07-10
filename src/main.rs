use esp_idf_svc::hal::{
    i2c::{I2cDriver, I2cConfig},
    gpio::PinDriver,
    peripherals::Peripherals,
    prelude::FromValueType,
};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::sys::TickType_t;
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use axp192::{Axp192, Axp192Error};

// AXP192のI2Cアドレス
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

    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();

    // I2C初期化
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    info!("Initializing I2C driver...");
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()),
    )?;
    info!("I2C driver initialized successfully.");

    // AXP192が起動するまで少し待つ
    thread::sleep(Duration::from_millis(50));

    // AXP192インスタンス生成
    let mut axp192 = Axp192::new(i2c);

    // I2Cドライバ取得用
    let i2c_ref = axp192.i2c_mut();

    // チップID読み出しは直接I2Cアクセス
    let mut chip_id_buf = [0u8; 1];
    i2c_ref.write_read(
        0x34,
        &[0x03],
        &mut chip_id_buf,
        100,
    )?;
    info!("AXP192 Chip ID: 0x{:X}", chip_id_buf[0]);

    // 割り込み許可
    axp192.enable_irq(AXP192_PEK_SHORT_PRESS_BIT)?;
    info!("AXP192 PEK IRQ enabled");

    // IRQステータス読み出しとクリア
    let irq_status = axp192.read_irq_status()?;
    if (irq_status & AXP192_PEK_SHORT_PRESS_BIT) != 0 {
        info!("PEK Short Press detected");
    }
    axp192.clear_irq_status(irq_status)?;

    info!("AXP192 IRQ status cleared!");

    // GPIO35の初期化
    info!("Initializing GPIO35 for button...");
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    info!("GPIO35 interrupt type set.");
    unsafe {
        button.subscribe(move || {
            let mut pressed = button_pressed_clone.lock().unwrap();
            *pressed = true;
            info!("GPIO35 interrupt triggered!");
        })
    }?;
    info!("GPIO35 subscribed to interrupts.");

    info!("Entering main loop...");

    loop {
        {
            let mut pressed = button_pressed.lock().unwrap();
            if *pressed {
                info!("Button Pressed (from ESP32 GPIO)!");
                *pressed = false;

                // 割り込みステータス読み出し
                let irq_status = axp192.read(AXP192_PEK_IRQ_STATUS1)?;
                if (irq_status & AXP192_PEK_SHORT_PRESS_BIT) != 0 {
                    info!("AXP192 PEK Short Press detected (from IRQ)!");
                }

                // ステータスクリア
                axp192.write(AXP192_PEK_IRQ_STATUS1, irq_status)?;
                info!("AXP192 IRQ status cleared.");
                thread::sleep(Duration::from_millis(50));
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}
