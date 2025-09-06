// src/drivers/touch.rs
use anyhow::Result;
use ft6x36::{Ft6x36, RawTouchEvent, Dimension, TouchType};
use crate::manager::I2cManager;

pub struct Touch<'a> {
    driver: Ft6x36<&'a I2cManager>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TouchEvent {
    Press,
    Release,
    Move,
}

pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub event: TouchEvent,
}

impl<'a> Touch<'a> {
    pub fn new(i2c: &'a I2cManager) -> Result<Self> {
        let driver = Ft6x36::new(i2c, Dimension(240, 240));
        Ok(Self { driver })
    }
}

impl<'a> Touch<'a>
{
    pub fn read_event(&mut self) -> Result<Option<TouchPoint>> {
        let raw_event: RawTouchEvent = self.driver.get_touch_event().map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(ft6x36_touch_point) = raw_event.p1 {
            let touch_event = match ft6x36_touch_point.touch_type {
                TouchType::Press => TouchEvent::Press,
                TouchType::Release => TouchEvent::Release,
                _ => return Ok(None),
            };

            return Ok(Some(TouchPoint {
                x: ft6x36_touch_point.x,
                y: ft6x36_touch_point.y,
                event: touch_event,
            }));
        }

        Ok(None)
    }
}

impl TouchPoint {
    pub fn on_button1(&self) -> bool {
        self.x < 100 && self.y < 100
    }

    pub fn on_button2(&self) -> bool {
        self.x > 140 && self.y < 100
    }

    pub fn on_back(&self) -> bool {
        self.y > 200
    }
}