// src/drivers/touch.rs
use anyhow::Result;
use ft6x36::{Ft6x36, RawTouchEvent, Dimension, TouchType};
use crate::manager::I2cManager;

pub struct Touch;

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

impl Touch {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl Touch
{
    pub fn read_event(&mut self, i2c: &I2cManager) -> Result<Option<TouchPoint>, anyhow::Error> {
        let mut driver = Ft6x36::new(i2c, Dimension(240, 240));
        let raw_event: RawTouchEvent = driver.get_touch_event().map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(ft6x36_touch_point) = raw_event.p1 {
            let touch_event = match ft6x36_touch_point.touch_type {
                TouchType::Press => TouchEvent::Press,
                TouchType::Release => TouchEvent::Release,
                TouchType::Contact => TouchEvent::Move,
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