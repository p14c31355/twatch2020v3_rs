mod axp192;

use esp_idf_svc::hal::{
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::FromValueType,
    gpio::PinDriver,
};
use esp_idf_svc::hal::delay::FreeRtos;
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use axp192::Axp192;

const AXP192_ADDR: u8 = 0x34;
const PEK_IRQ_BIT: u8 = 0x02;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Application started.");

    let peripherals = Peripherals::take().unwrap();
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(400u32.kHz().into()),
    )?;

    let mut axp192 = Axp192::new(i2c, AXP192_ADDR);

    // チップID確認
    let chip_id = axp192.read_reg(0x03)?;
    info!("AXP192 Chip ID: 0x{:X}", chip_id);

    // PEKボタン割り込み許可
    axp192.enable_irq(PEK_IRQ_BIT)?;
    info!("PEK IRQ enabled");

    // ボタン割り込み検知用のフラグ
    let button_pressed = Arc::new(Mutex::new(false));
    let button_pressed_clone = button_pressed.clone();

    // GPIO35 割り込み設定
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;
    button.set_interrupt_type(esp_idf_svc::hal::gpio::InterruptType::NegEdge)?;
    unsafe {
        button.subscribe(move || {
            let mut pressed = button_pressed_clone.lock().unwrap();
            *pressed = true;
        })
    }?;

    info!("GPIO35 interrupt subscribed.");

    // メインループ
    loop {
        {
            let mut pressed = button_pressed.lock().unwrap();
            if *pressed {
                *pressed = false;

                // 割り込みステータス取得
                let irq_status = axp192.read_irq_status()?;
                if (irq_status & PEK_IRQ_BIT) != 0 {
                    info!("AXP192 PEK Short Press detected!");
                }

                // 割り込みステータスクリア
                axp192.clear_irq_status(irq_status)?;

                thread::sleep(Duration::from_millis(50));
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}
