use esp_idf_hal::gpio::PinDriver;
use esp_idf_sys::EspError;
use log::info;
use async_button::*;
use async_std;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = esp_idf_hal::prelude::Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;
    let mut button = Button::new(pin, ButtonConfig::default());

    async_std::task::spawn(async move {
        loop {
            match button.update().await {
                ButtonEvent::ShortPress { count: _ } => info!("HelloButton!"),
                ButtonEvent::LongPress => info!("It hurts!"),
            }
        }
    });

    loop{
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

}