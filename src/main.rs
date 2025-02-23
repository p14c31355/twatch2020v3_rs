use esp_idf_svc::timer::EspTimer;
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys::EspError;
use log::info;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;
    let mut last_state = pin.is_high();
    let debounce_delay = Duration::from_millis(50);
    let mut last_click_time = EspTimer::new()?;
    let double_click_interval = Duration::from_millis(300);
    let mut click_count = 0;

    loop {
        let current_state = pin.is_low();
        if last_state != current_state {
            thread::sleep(debounce_delay);
            if pin.is_low() == current_state {
                last_state = current_state;
                if current_state {
                    let now = EspTimer::new()?;
                    let elapsed = now.elapsed() - last_click_time.elapsed();
                    if elapsed < double_click_interval{
                        click_count += 1;
                        info!("double click");
                    }else{
                        click_count = 1;
                        info!("click");
                    }
                    last_click_time = now;
                }else{
                    info!("button released");
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}