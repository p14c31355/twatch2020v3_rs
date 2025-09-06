// src/drivers/touch.rs
use anyhow::Result;
use ft6x36::{Ft6x36, TouchEvent, RawTouchEvent, Dimension}; 

pub struct Touch<'a, I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    driver: Ft6x36<I2C>,
    _phantom: core::marker::PhantomData<&'a ()>,
}

pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub event: TouchEvent,
}

impl<'a> Touch<'a, &'a mut esp_idf_hal::i2c::I2cDriver<'a>>
{
    pub fn new_with_ref(i2c_ref: &'a mut esp_idf_hal::i2c::I2cDriver<'a>) -> Result<Self> {
        let driver = Ft6x36::new(i2c_ref, Dimension(240, 240));
        Ok(Self {
            driver,
            _phantom: core::marker::PhantomData,
        })
    }

    pub fn read_event(&mut self) -> Result<Option<TouchPoint>> {
        let raw_event: RawTouchEvent = self.driver.get_touch_event().map_err(|e| anyhow::anyhow!("{:?}", e))?;

        if let Some(ft6x36_touch_point) = raw_event.into() {
            return Ok(Some(TouchPoint {
                x: ft6x36_touch_point.x,
                y: ft6x36_touch_point.y,
                event: ft6x36_touch_point.event,
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