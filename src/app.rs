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
use esp_idf_hal::task::watchdog::{TWDTDriver, TWDTConfig};
use chrono::{Local, Timelike};
use std::time::Duration;

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
    twdt: TWDTDriver<'a>,
}

impl<'a> App<'a> {
    pub fn new(
        i2c: I2cManager,
        display: TwatchDisplay<'a>,
        power: PowerManager,
        touch: Touch,
        twdt: esp_idf_hal::task::watchdog::TWDT,
    ) -> Result<Self> {
        let config = TWDTConfig {
            duration: Duration::from_secs(30),
            panic_on_trigger: true,
            subscribed_idle_tasks: Default::default(),
        };
        let driver = TWDTDriver::new(twdt, &config)?;
        Ok(Self {
            i2c,
            display,
            power,
            touch,
            state: AppState::Launcher,
            twdt: driver,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.power.init_power(&mut self.i2c)?;
        self.power.set_backlight(&mut self.i2c, true)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.feed_watchdog()?;          // WDT feed
            self.draw_status_bar()?;        // 状態バー描画
            self.handle_state()?;           // UI状態遷移処理
            self.feed_and_delay(20)?;       // 微小ディレイ + WDT feed
        }
    }

    fn handle_state(&mut self) -> Result<()> {
        match self.state.clone() {
            AppState::Launcher => self.show_launcher()?,
            AppState::Settings => self.show_settings()?,
            AppState::Battery => self.show_battery()?,
        }
        Ok(())
    }

    fn show_launcher(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        draw_text(&mut self.display.display, "Launcher: tap for apps", 10, 40)?;

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            self.state = if event.on_button1() {
                AppState::Settings
            } else if event.on_button2() {
                AppState::Battery
            } else {
                self.state.clone()
            };
        }
        Ok(())
    }

    fn show_settings(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        draw_text(&mut self.display.display, "Settings", 10, 40)?;

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }
        Ok(())
    }

    fn show_battery(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);

        let voltage = self.power.read_voltage(&mut self.i2c)?;
        draw_text(
            &mut self.display.display,
            &format!("Battery: {voltage} mV"),
            10,
            40,
        )?;

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }
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

        let battery = self.power.get_battery_percentage(&mut self.i2c)?;
        draw_text(&mut self.display.display, &format!("{battery}%"), 200, 10)?;
        Ok(())
    }

    /// WDT feed
    fn feed_watchdog(&mut self) -> Result<()> {
        // task_subscriptionを一時作成して feed して即 drop
        let mut task_watch = self.twdt.watch_current_task()?;
        task_watch.feed();
        Ok(())
    }

    fn feed_and_delay(&mut self, ms: u32) -> Result<()> {
        let mut remaining = ms;
        while remaining > 0 {
            let step = remaining.min(10);
            FreeRtos::delay_ms(step);
            self.feed_watchdog()?;
            remaining -= step;
        }
        Ok(())
    }
}

/// 共通テキスト描画
fn draw_text<D: DrawTarget<Color = Rgb565>>(
    target: &mut D,
    content: &str,
    x: i32,
    y: i32,
) -> Result<()> {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    Text::new(content, Point::new(x, y), style).draw(target);
    Ok(())
}
