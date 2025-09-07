// src/app.rs
use anyhow::Result;
use crate::{
    drivers::{axp::PowerManager, display::TwatchDisplay, touch::Touch},
    manager::I2cManager,
};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
    Drawable,
};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::delay::Delay;
use chrono::{NaiveTime, Timelike};

const DELAY_MS: u32 = 100;
pub fn feed_watchdog() {
    // Note: This is a delay, not a true watchdog feed. If a watchdog is enabled, it needs to be fed.
    FreeRtos::delay_ms(DELAY_MS);
}


#[derive(Debug)]
pub enum AppState {
    Launcher,
    Settings,
    Battery,
}

pub struct App<'a> {
    i2c: I2cManager,
    display: TwatchDisplay<'a>,
    power: PowerManager,
    touch: Touch,
    state: AppState,
}

impl<'a> App<'a> {
    pub fn new(i2c: I2cManager, display: TwatchDisplay<'a>, power: PowerManager, touch: Touch) -> Self {
        Self {
            i2c,
            display,
            power,
            touch,
            state: AppState::Launcher,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.power.init_power(&mut self.i2c)?;
        self.power.set_backlight(&mut self.i2c, true)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.draw_status_bar()?;
            feed_watchdog();

            match self.state {
                AppState::Launcher => self.show_launcher()?,
                AppState::Settings => self.show_settings()?,
                AppState::Battery => self.show_battery()?,
            }
            feed_watchdog();
        }
    }

    fn show_launcher(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        feed_watchdog();

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        Text::new("Launcher: tap for apps", Point::new(10, 40), text_style)
            .draw(&mut self.display.display);
        feed_watchdog();

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            if event.on_button1() {
                self.state = AppState::Settings;
            } else if event.on_button2() {
                self.state = AppState::Battery;
            }
        }

        feed_watchdog();
        FreeRtos::delay_ms(20);
        Ok(())
    }

    fn show_settings(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        feed_watchdog();

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        Text::new("Settings", Point::new(10, 40), text_style)
            .draw(&mut self.display.display);
        feed_watchdog();

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }

        feed_watchdog();
        FreeRtos::delay_ms(20);
        Ok(())
    }

    fn show_battery(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        feed_watchdog();

        let voltage = self.power.read_voltage(&mut self.i2c)?;
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        Text::new(&format!("Battery: {voltage} mV"), Point::new(10, 40), text_style)
            .draw(&mut self.display.display);
        feed_watchdog();

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }

        feed_watchdog();
        FreeRtos::delay_ms(20);
        Ok(())
    }

    fn draw_status_bar(&mut self) -> Result<()> {
        let now = NaiveTime::from_hms_opt(12, 34, 56).unwrap();
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

        let time_str = format!("{:02}:{:02}", now.hour(), now.minute());
        Text::new(&time_str, Point::new(10, 10), text_style)
            .draw(&mut self.display.display);
        feed_watchdog();

        let battery_percentage = self.power.get_battery_percentage(&mut self.i2c)?;
        let battery_str = format!("{battery_percentage}%");
        Text::new(&battery_str, Point::new(200, 10), text_style)
            .draw(&mut self.display.display);
        feed_watchdog();

        FreeRtos::delay_ms(20);

        Ok(())
    }
}
