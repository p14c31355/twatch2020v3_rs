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
use esp_idf_hal::task::watchdog::{TWDTDriver, TWDTConfig, TWDTWatch};
use esp_idf_hal::peripherals::Peripherals;
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
    watchdog: TWDTWatch,
}

impl<'a> App<'a> {
    pub fn new(
        i2c: I2cManager,
        display: TwatchDisplay<'a>,
        power: PowerManager,
        touch: Touch,
        peripherals: &Peripherals,
    ) -> Result<Self> {
        // タスクウォッチドッグ設定
        let config = TWDTConfig {
            duration: Duration::from_secs(30), // 30秒
            panic_on_trigger: true,
            subscribed_idle_tasks: Default::default(),
        };
        let driver = TWDTDriver::new(peripherals.twdt, &config)?;
        let watchdog = driver.watch_current_task()?; // 現在のタスクを監視対象に

        Ok(Self {
            i2c,
            display,
            power,
            touch,
            state: AppState::Launcher,
            watchdog,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.power.init_power(&mut self.i2c)?;
        self.power.set_backlight(&mut self.i2c, true)?;
        self.feed_watchdog()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.feed_watchdog()?;
            self.draw_status_bar()?;
            self.handle_state()?;
            self.feed_and_delay(20)?;
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
        self.feed_watchdog()?;
        draw_text(&mut self.display.display, "Launcher: tap for apps", 10, 40, &mut self.watchdog)?;

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            self.state = if event.on_button1() {
                AppState::Settings
            } else if event.on_button2() {
                AppState::Battery
            } else {
                self.state.clone()
            };
        }
        self.feed_and_delay(20)?;
        Ok(())
    }

    fn show_settings(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        self.feed_watchdog()?;
        draw_text(&mut self.display.display, "Settings", 10, 40, &mut self.watchdog)?;

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }
        self.feed_and_delay(20)?;
        Ok(())
    }

    fn show_battery(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        self.feed_watchdog()?;

        let voltage = self.power.read_voltage(&mut self.i2c)?;
        draw_text(
            &mut self.display.display,
            &format!("Battery: {voltage} mV"),
            10,
            40,
            &mut self.watchdog,
        )?;

        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
            if event.on_back() {
                self.state = AppState::Launcher;
            }
        }
        self.feed_and_delay(20)?;
        Ok(())
    }

    fn draw_status_bar(&mut self) -> Result<()> {
        let now = Local::now();
        draw_text(
            &mut self.display.display,
            &format!("{:02}:{:02}", now.hour(), now.minute()),
            10,
            10,
            &mut self.watchdog,
        )?;

        let battery = self.power.get_battery_percentage(&mut self.i2c)?;
        draw_text(&mut self.display.display, &format!("{battery}%"), 200, 10, &mut self.watchdog)?;
        Ok(())
    }

    fn feed_watchdog(&mut self) -> Result<()> {
        self.watchdog.feed();
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

/// 共通テキスト描画 + WDT feed
fn draw_text<D: DrawTarget<Color = Rgb565>>(
    target: &mut D,
    content: &str,
    x: i32,
    y: i32,
    watchdog: &mut TWDTWatch,
) -> Result<()> {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    Text::new(content, Point::new(x, y), style).draw(target);
    watchdog.feed();
    Ok(())
}
