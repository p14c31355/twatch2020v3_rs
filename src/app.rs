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
use esp_idf_hal::task::watchdog::TWDTDriver;
use esp_idf_hal::task::watchdog::WatchdogSubscription;
use chrono::{Local, Timelike};

#[derive(Debug, Clone)]
pub enum AppState {
    Launcher,
    Settings,
    Battery,
}
// TWDTDriverのライフタイムをApp構造体と合わせる
pub struct App<'a, 'b> {
    i2c: I2cManager,
    display: TwatchDisplay<'a>,
    power: PowerManager,
    touch: Touch,
    state: AppState,
    twdt_driver: TWDTDriver<'a, 'b>, // TWDTDriverの所有権を保持
    twdt_subscription: WatchdogSubscription<'a>, // TWDTDriverのライフタイムに依存
}

impl<'a, 'b> App<'a, 'b> {
    pub fn new(
        mut i2c: I2cManager,
        mut display: TwatchDisplay<'a>,
        mut power: PowerManager, // powerのライフタイムをApp構造体と合わせる
        mut touch: Touch,
        mut twdt_driver: TWDTDriver, // TWDTDriverの所有権を取得
    ) -> Result<Self> {
        let mut twdt_subscription = twdt_driver.watch_current_task()?;
        twdt_subscription.feed();
        FreeRtos::delay_ms(10);

        power.init_power(&mut i2c)?;
        twdt_subscription.feed();
        FreeRtos::delay_ms(10);

        power.set_backlight(&mut i2c, true)?;
        twdt_subscription.feed();
        FreeRtos::delay_ms(10);

        display.display.clear(Rgb565::BLACK);
        twdt_subscription.feed();
        FreeRtos::delay_ms(10);

        Ok(Self {
            i2c,
            display,
            power,
            touch,
            state: AppState::Launcher,
            twdt_driver, // twdt_driverフィールドを初期化
            twdt_subscription,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.twdt_subscription.feed();
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
        draw_text(&mut self.display.display, "Launcher: tap for apps", 10, 40)?;
        if let Some(event) = self.touch.read_event(&mut self.i2c)? {
                        if event.on_button1() {
                self.state = AppState::Settings;
            } else if event.on_button2() {
                self.state = AppState::Battery;
            }
        }
        self.feed_and_delay(20)?;
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
        self.feed_and_delay(20)?;
        Ok(())
    }

    fn show_battery(&mut self) -> Result<()> {
        self.display.display.clear(Rgb565::BLACK);
        let voltage = self.power.read_voltage(&mut self.i2c)?;
        draw_text(&mut self.display.display, &format!("Battery: {voltage} mV"), 10, 40)?;
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
        draw_text(&mut self.display.display, &format!("{:02}:{:02}", now.hour(), now.minute()), 10, 10)?;
        let battery = self.power.get_battery_percentage(&mut self.i2c)?;
        draw_text(&mut self.display.display, &format!("{battery}%"), 200, 10)?;
        self.twdt_subscription.feed();
        Ok(())
    }

    fn feed_and_delay(&mut self, ms: u32) -> Result<()> {
        let mut remaining = ms;
        while remaining > 0 {
            let step = remaining.min(10);
            FreeRtos::delay_ms(step);
            self.twdt_subscription.feed();
            remaining -= step;
        }
        Ok(())
    }
}

fn draw_text<D: DrawTarget<Color = Rgb565>>(target: &mut D, content: &str, x: i32, y: i32) -> Result<()>
where
    D::Error: std::fmt::Debug,
{
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    Text::new(content, Point::new(x, y), style)
        .draw(target)
        .map_err(|e| anyhow::Error::msg(format!("{e:?}")))?;
    Ok(())
}
