// src/drivers/touch.rs
use crate::manager::I2cManager;
use anyhow::Result;
use ft6x36::{Dimension, Ft6x36, RawTouchEvent, TouchType};

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

pub struct Touch {
    driver: Ft6x36<I2cManager>,
}

impl Touch {
    pub fn new(i2c: I2cManager) -> Result<Self> {
        let driver = Ft6x36::new(i2c, Dimension(240, 240));
        Ok(Self { driver })
    }

    pub fn read_event(&mut self) -> Result<Option<TouchPoint>> {
        let raw_event: RawTouchEvent = self.driver
            .get_touch_event()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(p) = raw_event.p1 {
            let event = match p.touch_type {
                TouchType::Press => TouchEvent::Press,
                TouchType::Release => TouchEvent::Release,
                TouchType::Contact => TouchEvent::Move,
                _ => return Ok(None),
            };

            Ok(Some(TouchPoint { x: p.x, y: p.y, event }))
        } else {
            Ok(None)
        }
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
