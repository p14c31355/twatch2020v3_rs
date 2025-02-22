#[warn(unused_imports)]
use button_driver::{Button, ButtonConfig};
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys::EspError;
use log::info;
use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::Input;
use button_driver::PinWrapper;

/*
impl Button for Instant {
    fn new(&self) {        

    }
}
*/

struct MyPinWrapper<'a, T: InputPin> {
    pin: PinDriver<'a, T, Input>,
}

//PinWrapperトレイトをMyPinWrapperに実装する。
impl<'a, T: InputPin> PinWrapper for MyPinWrapper<'a, T> {
    //PinWrapperトレイトのメソッドを実装する。
    fn is_high(&mut self) -> bool {
        self.pin.is_high().unwrap_or(false)
    }
    //他のPinWrapperトレイトのメソッドを実装する。
}

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio35)?;
    let my_pin = MyPinWrapper { pin };
    let mut button: button_driver::Button<MyPinWrapper<'_, Gpio35>, I> = Button::new(my_pin, ButtonConfig::default());

loop {
    button.tick();
     
    if button.is_clicked() {
        println!("Clicked!");
    }

    button.reset();

    }
}