use esp_idf_hal::gpio::PinDriver;
use esp_idf_sys::EspError;
use log::info;
use async_button::*;
use async_std;
use std::sync::{Arc, Mutex};

struct ThreadSafeButton {
    button: Arc<Mutex<Button<PinDriver<'static, esp_idf_hal::gpio::Gpio35, esp_idf_hal::gpio::Input>>>>,
}

impl ThreadSafeButton {
    fn new(button: Button<PinDriver<'static, esp_idf_hal::gpio::Gpio35, esp_idf_hal::gpio::Input>>) -> Self {
        ThreadSafeButton {
            button: Arc::new(Mutex::new(button)),
        }
    }

    async fn update(&self) -> ButtonEvent {
        self.button.lock().unwrap().update().await
    }
}

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = esp_idf_hal::prelude::Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35).unwrap();
    let button = Button::new(pin, ButtonConfig::default());
    let safe_button = ThreadSafeButton::new(button);

    async_std::task::spawn(async move {
        loop {
            match safe_button.update().await {
                ButtonEvent::ShortPress { count: _ } => info!("HelloButton!"),
                ButtonEvent::LongPress => info!("It hurts!"),
            }
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}