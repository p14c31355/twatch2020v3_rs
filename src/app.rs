// src/app.rs
use crate::{
    drivers::{axp::PowerManager, display::TwatchDisplay, touch::Touch},
    manager::I2cManager,
};
use anyhow::Result;
use chrono::{Local, Timelike};
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use esp_idf_hal::delay::FreeRtos;

pub fn feed_watchdog() {
    unsafe { esp_idf_sys::esp_task_wdt_reset() };
}

#[derive(Debug, Clone)]
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
    pub fn new(
        i2c: I2cManager,
        display: TwatchDisplay<'a>,
        mut power: PowerManager,
        touch: Touch,
    ) -> Result<Self> {
        power.set_backlight(true)?;

        Ok(Self {
            i2c,
            display,
            power,
            touch,
            state: AppState::Launcher,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            feed_watchdog();
            self.draw_status_bar()?;

            if let Some(tp) = self.touch.read_event()? {
                self.handle_touch(tp)?;
            }

            self.handle_state()?;
            self.feed_and_delay(20)?;
        }
    }

    fn handle_touch(&mut self, tp: crate::drivers::touch::TouchPoint) -> Result<()> {
        match self.state {
            AppState::Launcher => {
                if tp.on_button1() {
                    self.state = AppState::Settings;
                } else if tp.on_button2() {
                    self.state = AppState::Battery;
                }
            }
            AppState::Settings | AppState::Battery => {
                if tp.on_back() {
                    self.state = AppState::Launcher;
                }
            }
        }
        Ok(())
    }

    fn handle_state(&mut self) -> Result<()> {
        match self.state {
            AppState::Launcher => self.show_launcher()?,
            AppState::Settings => self.show_settings()?,
            AppState::Battery => self.show_battery()?,
        }
        Ok(())
    }

    fn show_launcher(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        draw_text(&mut self.display.display, "Launcher: tap for apps", 10, 40)?;
        Ok(())
    }

    fn show_settings(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        draw_text(&mut self.display.display, "Settings", 10, 40)?;
        Ok(())
    }

    fn show_battery(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        let voltage = self.power.read_voltage()?;
        draw_text(
            &mut self.display.display,
            &format!("Battery: {voltage} mV"),
            10,
            40,
        )?;
        Ok(())
    }

    fn draw_status_bar(&mut self) -> Result<()> {
        let now = Local::now();
        draw_text(
            &mut self.display.display,
            &format!("{:02}:{:02}", now.hour(), now.minute()),
            10,
            10,
        )?;
        let battery = self.power.get_battery_percentage()?;
        draw_text(&mut self.display.display, &format!("{battery}%"), 200, 10)?;
        Ok(())
    }

    fn feed_and_delay(&mut self, ms: u32) -> Result<()> {
        let mut remaining = ms;
        while remaining > 0 {
            let step = remaining.min(10);
            FreeRtos::delay_ms(step);
            feed_watchdog();
            remaining -= step;
        }
        Ok(())
    }
}

fn draw_text<D: DrawTarget<Color = Rgb565>>(
    target: &mut D,
    content: &str,
    x: i32,
    y: i32,
) -> Result<()>
where
    D::Error: std::fmt::Debug,
{
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    Text::new(content, Point::new(x, y), style)
        .draw(target)
        .map_err(|e| anyhow::Error::msg(format!("{e:?}")))?;
    Ok(())
}
