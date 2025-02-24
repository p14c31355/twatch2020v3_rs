use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_sys::*;
use log::*;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    // Gpio35ピンを入力モードで内部プルアップを有効にして設定
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;

    unsafe {
        gpio_set_intr_type(button.pin(), gpio_int_type_t_GPIO_INTR_NEGEDGE);
        gpio_intr_enable(1 << button.pin());
        gpio_isr_handler_add(button.pin(), Some(isr_handler), std::ptr::null_mut());
    }

    extern "C" fn isr_handler(_: *mut std::ffi::c_void) {
        info!("Button pressed!");
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    Ok(())
}