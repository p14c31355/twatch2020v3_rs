// src/app.rs
use anyhow::Result;
use crate::drivers::{axp::PowerManager, display::TwatchDisplay, touch::Touch};
use embedded_graphics::{pixelcolor::Rgb565,
    Drawable, mono_font::{MonoTextStyle, ascii::FONT_6X10}, prelude::{DrawTarget, Point, RgbColor}, text::Text,
}; 
use esp_idf_hal::delay::FreeRtos;
use chrono::{NaiveTime, Timelike}; // Add this line

#[derive(Debug)]
pub enum AppState {
    Launcher,
    Settings,
    Battery,
}

pub struct App<'a> {
    power_manager: PowerManager<'a>,
    touch: Touch<'a>,
    display: TwatchDisplay<'a>,
    state: AppState,
}

impl<'a> App<'a> {
    pub fn new(power_manager: PowerManager<'a>, touch: Touch<'a>, display: TwatchDisplay<'a>) -> Self {
        Self { power_manager, touch, display, state: AppState::Launcher }
    }

    pub fn run(&mut self, _delay: &mut FreeRtos) -> Result<()> {
        loop {
            self.draw_status_bar()?;
            match self.state {
                AppState::Launcher => self.show_launcher()?,
                AppState::Settings => self.show_settings()?,
                AppState::Battery => self.show_battery()?,
            }
        }
    }

    fn draw_status_bar(&mut self) -> Result<()> {
        let now = NaiveTime::from_hms_opt(12, 34, 0).unwrap(); // Placeholder for actual time        
        let battery = self.power_manager.read_voltage()?;
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        Text::new(&format!("{:02}:{:02}", now.hour(), now.minute()), Point::new(5, 5), text_style)
            .draw(&mut self.display.display);
        Text::new(&format!("{:.0}%", battery), Point::new(180, 5), text_style)
            .draw(&mut self.display.display);

        Ok(())
    }

    fn show_launcher(&mut self) -> Result<()> {
            let mut touch = &mut self.touch;
            if let Some(event) = touch.read_event()? {
                if event.on_button1() { self.state = AppState::Settings; }
                else if event.on_button2() { self.state = AppState::Battery; }
            }
        self.display.display.clear(Rgb565::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        Text::new("Launcher: tap for apps", Point::new(10, 40), text_style)
            .draw(&mut self.display.display);
        FreeRtos::delay_ms(50);
        Ok(())
    }

    fn show_settings(&mut self) -> Result<()> {
            let mut touch = &mut self.touch;
            if let Some(event) = touch.read_event()? { // Pass a mutable reference
                if event.on_back() { self.state = AppState::Launcher; }
            }
        self.display.display.clear(Rgb565::BLACK);
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        Text::new("Settings", Point::new(10, 40), text_style)
            .draw(&mut self.display.display);
        FreeRtos::delay_ms(50);
        Ok(())
    }

    fn show_battery(&mut self) -> Result<()> {
            let voltage = self.power_manager.read_voltage()?;
            self.display.display.clear(Rgb565::BLACK);
            let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
            Text::new(&format!("Battery: {:.2} V", voltage / 1000.0), Point::new(10, 40), text_style)
                .draw(&mut self.display.display);

        let mut touch = &mut self.touch;
        if let Some(event) = touch.read_event()? { // Pass a mutable reference
            if event.on_back() { self.state = AppState::Launcher; }
        }
        FreeRtos::delay_ms(200);
        Ok(())
    }
}
