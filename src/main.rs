use esp_idf_hal::timer::{Timer, TimerDriver, config::Config};
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys::EspError;
use log::info;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio9)?;
    let mut last_state = pin.is_high();
    let debounce_delay = Duration::from_millis(50);

    let timer = <dyn Timer>::new(peripherals.timer00)?;
    let mut timer_driver = TimerDriver::new(timer, &Config::default())?;
    timer_driver.enable(true)?; // trueを渡してタイマーを有効にする

    let mut last_click_time = timer_driver.counter()?;
    let double_click_interval = Duration::from_millis(300);
    let mut click_count = 0;

    loop {
        let current_state = pin.is_low();
        if last_state != current_state {
            thread::sleep(debounce_delay);
            if pin.is_low() == current_state {
                last_state = current_state;
                if current_state {
                    let now = timer_driver.counter()?;
                    let elapsed = now - last_click_time;
                    if elapsed < double_click_interval.as_micros() as u64 {
                        click_count += 1;
                        info!("double click");
                    } else {
                        click_count = 1;
                        info!("click");
                    }
                    last_click_time = now;
                } else {
                    info!("button released");
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}