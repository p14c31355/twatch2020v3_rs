// src/drivers/axp.rs
use anyhow::Result;
use esp_idf_hal::delay::FreeRtos;
use axp20x::{Axpxx, Power, PowerState};
use crate::manager::I2cManager;

pub struct PowerManager {
    pub axp: Axpxx<I2cManager>,
}

impl<'a> PowerManager {
    pub fn new(i2c: I2cManager) -> Result<Self> {
        let mut axp = Axpxx::new(i2c);
        axp.init().map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(Self { axp })
    }

    pub fn backlight_on(&mut self, delay: &mut FreeRtos) -> Result<()> {
        self.axp
            .set_power_output(Power::Ldo2, PowerState::On, delay)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }

    pub fn backlight_off(&mut self, delay: &mut FreeRtos) -> Result<()> {
        self.axp
            .set_power_output(Power::Ldo2, PowerState::Off, delay)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }

    pub fn read_voltage(&mut self) -> Result<f32> {
        self.axp.get_battery_voltage().map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}