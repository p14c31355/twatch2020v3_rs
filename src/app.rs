// src/app.rs
use anyhow::Result;
use crate::drivers::{axp::PowerManager, display::MyDisplay, touch::Touch};
use esp_idf_hal::delay::FreeRtos;

#[derive(Debug)]
pub enum AppState {
    Home,
    Settings,
    Battery,
}

pub struct App<'a> {
    power: PowerManager<'a>,
    display: MyDisplay<'a>,
    touch: Touch<'a>,
    state: AppState,
}

impl<'a> App<'a> {
    pub fn new(power: PowerManager<'a>, display: MyDisplay<'a>, touch: Touch<'a>) -> Self {
        Self {
            power,
            display,
            touch,
            state: AppState::Home,
        }
    }

    pub fn run(&mut self, delay: &mut FreeRtos) -> Result<()> {
        loop {
            match self.state {
                AppState::Home => self.show_home(delay)?,
                AppState::Settings => self.show_settings(delay)?,
                AppState::Battery => self.show_battery(delay)?,
            }
        }
    }

    fn show_home(&mut self, delay: &mut FreeRtos) -> Result<()> {
        self.display.clear()?;
        self.display.draw_text(10, 20, "Home Screen")?;

        if let Some(event) = self.touch.read_event()? {
            if event.on_button1() {
                self.state = AppState::Settings;
            } else if event.on_button2() {
                self.state = AppState::Battery;
            }
        }

        delay.delay_ms(50);
        Ok(())
    }

    fn show_settings(&mut self, delay: &mut FreeRtos) -> Result<()> {
        self.display.clear()?;
        self.display.draw_text(10, 20, "Settings")?;
        if let Some(event) = self.touch.read_event()? {
            if event.on_back() {
                self.state = AppState::Home;
            }
        }
        delay.delay_ms(50);
        Ok(())
    }

    fn show_battery(&mut self, delay: &mut FreeRtos) -> Result<()> {
        self.display.clear()?;
        let voltage = self.power.read_voltage()?;
        self.display.draw_text(10, 20, &format!("Battery: {} mV", voltage))?;
        if let Some(event) = self.touch.read_event()? {
            if event.on_back() {
                self.state = AppState::Home;
            }
        }
        delay.delay_ms(200);
        Ok(())
    }
}
