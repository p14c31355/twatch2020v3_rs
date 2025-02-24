use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_sys::*;
use log::*;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mut button = PinDriver::input(peripherals.pins.gpio35)?;

    // Gpio35は入力専用ピンのため、set_pullは使用できません。
    // 外部回路でプルアップ抵抗を有効化してください。
    // または、内部プルアップを有効化する場合は以下のようにします。
    button.set_pull(Pull::Up)?;

    unsafe {
        gpio_set_intr_type(button.pin(), gpio_int_type_t_GPIO_INTR_NEGEDGE);
        gpio_intr_enable(1 << button.pin());
        gpio_isr_handler_add(button.pin(), Some(isr_handler), std::ptr::null_mut())?;
    }

    extern "C" fn isr_handler(_: *mut std::ffi::c_void) {
        info!("Button pressed!");
    }

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}