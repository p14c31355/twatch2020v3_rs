use anyhow::Result;
use crate::{drivers::{axp::PowerManager, display::TwatchDisplay, touch::Touch}, manager::I2cManager};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    text::Text,
    pixelcolor::Rgb565,
    Drawable,
    prelude::{Point, DrawTarget, RgbColor},
};
use esp_idf_hal::delay::FreeRtos;
use chrono::{NaiveTime, Timelike};

#[derive(Debug)]
pub enum AppState {
    Launcher,
    Settings,
    Battery,
}

pub struct App {
    power: PowerManager,
    display: TwatchDisplay<'static>,
    touch: Touch,
    state: AppState,
    i2c: I2cManager,
}


impl<'a> App {
    pub fn new(i2c: I2cManager, display: TwatchDisplay<'static>) -> Self {
        let power = PowerManager::new(i2c.clone()).unwrap();
        let touch = Touch::new(i2c.clone()).unwrap();

        Self {
            power,
            display,
            touch,
            state: AppState::Launcher,
            i2c,
        }
    }

    pub fn run(&mut self, _delay: &mut FreeRtos) -> Result<()> {
        loop {
            self.draw_status_bar()?;

            match self.state {
                AppState::Launcher => self.show_launcher()?,
                AppState::Settings => self.show_settings()?,
                AppState::Battery  => self.show_battery()?,
            }
        }
    }

    fn draw_status_bar(&mut self) -> Result<()> {
        let now = NaiveTime::from_hms_opt(12, 34, 0).unwrap();
        let battery = self.power.axp.get_battery_percentage()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        Text::new(&format!("{:02}:{:02}", now.hour(), now.minute()), Point::new(5, 5), text_style)
            .draw(&mut self.display.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Text::new(&format!("{}%", battery), Point::new(180, 5), text_style)
            .draw(&mut self.display.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(())
    }

    fn show_launcher(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        Text::new("Launcher: tap for apps", Point::new(10, 40), text_style)
            .draw(&mut self.display.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(event) = self.touch.read_event()? {
            if event.on_button1() {
                self.state = AppState::Settings;
            } else if event.on_button2() {
                self.state = AppState::Battery;
            }
        }
        FreeRtos::delay_ms(50);
        Ok(())
    }

    fn show_settings(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        Text::new("Settings", Point::new(10, 40), text_style)
            .draw(&mut self.display.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(event) = self.touch.read_event()? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }
        FreeRtos::delay_ms(50);
        Ok(())
    }

    fn show_battery(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let voltage = self.power.read_voltage()?;
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        Text::new(&format!("Battery: {} mV", voltage), Point::new(10, 40), text_style)
            .draw(&mut self.display.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(event) = self.touch.read_event()? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }
        FreeRtos::delay_ms(200);
        Ok(())
    }
}
