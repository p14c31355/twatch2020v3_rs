// src/app.rs
use anyhow::Result;
use crate::drivers::{axp::PowerManager, display::TwatchDisplay, touch::Touch};
use embedded_graphics::{mono_font::{MonoTextStyle, ascii::FONT_6X10}, text::Text, pixelcolor::Rgb565, Drawable, prelude::{Point, DrawTarget}};
use esp_idf_hal::{delay::FreeRtos, i2c::{I2C0, I2cError}};
use chrono::NaiveTime;

#[derive(Debug)]
pub enum AppState {
    Launcher,
    Settings,
    Battery,
}

pub struct App<'a, I2C, E>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
    E: core::fmt::Debug,
{
    power: PowerManager<'a>,
    display: TwatchDisplay<'a>,
    touch: Touch<'a, I2C>,
    state: AppState,
}

impl<'a, I2C, E> App<'a, I2C, E>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(power: PowerManager<'a>, display: TwatchDisplay<'a>, touch: Touch<'a, I2C>) -> Self {
        Self {
            power,
            display,
            touch,
            state: AppState::Launcher,
        }
    }

    pub fn run(&mut self, delay: &mut FreeRtos) -> Result<()> {
        loop {
            self.draw_status_bar()?;

            match self.state {
                AppState::Launcher => self.show_launcher(delay)?,
                AppState::Settings => self.show_settings(delay)?,
                AppState::Battery  => self.show_battery(delay)?,
            }
        }
    }

    fn draw_status_bar(&mut self) -> Result<()> {
        let now = NaiveTime::from_hms_opt(12, 34, 0).unwrap();

        let battery = self.power.axp.get_battery_percentage().map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let text_style = MonoTextStyle::new(&FONT_6X10, <Rgb565 as embedded_graphics::prelude::RgbColor>::WHITE);

        Text::new(&format!("{:02}:{:02}", chrono::Timelike::hour(&now), chrono::Timelike::minute(&now)), Point::new(5, 5), text_style).draw(&mut self.display).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Text::new(&format!("{}%", battery), embedded_graphics::geometry::Point::new(180, 5), text_style).draw(&mut self.display).map_err(|e| anyhow::anyhow!("{:?}", e))?;


        Ok(())
    }

    fn show_launcher(&mut self, delay: &mut FreeRtos) -> Result<()> {
        self.display.clear(<Rgb565 as embedded_graphics::prelude::RgbColor>::BLACK).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let text_style = MonoTextStyle::new(&FONT_6X10, <Rgb565 as embedded_graphics::prelude::RgbColor>::WHITE);
        Text::new("Launcher: tap for apps", embedded_graphics::geometry::Point::new(10, 40), text_style).draw(&mut self.display).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        
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

    fn show_settings(&mut self, delay: &mut FreeRtos) -> Result<()> {
        self.display.clear(<Rgb565 as embedded_graphics::prelude::RgbColor>::BLACK).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let text_style = MonoTextStyle::new(&FONT_6X10, <Rgb565 as embedded_graphics::prelude::RgbColor>::WHITE);
        Text::new("Settings", Point::new(10, 40), text_style).draw(&mut self.display).map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(event) = self.touch.read_event()? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }
        FreeRtos::delay_ms(50);
        Ok(())
    }

    fn show_battery(&mut self, delay: &mut FreeRtos) -> Result<()> {
        self.display.clear(<Rgb565 as embedded_graphics::prelude::RgbColor>::BLACK).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let voltage = self.power.read_voltage()?;
        let text_style = MonoTextStyle::new(&FONT_6X10, <Rgb565 as embedded_graphics::prelude::RgbColor>::WHITE); // This line was already present
        Text::new(&format!("Battery: {} mV", voltage), embedded_graphics::geometry::Point::new(10, 40), text_style).draw(&mut self.display).map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(event) = self.touch.read_event()? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }
        FreeRtos::delay_ms(200);
        Ok(())
    }
}
