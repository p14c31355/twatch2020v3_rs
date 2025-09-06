// src/drivers/axp.rs
use anyhow::Result;
use axp20x::{Axpxx, Power, PowerState};
use crate::manager::I2cManager;
use esp_idf_hal::delay::FreeRtos;

pub struct PowerManager<'a> {
    pub axp: Axpxx<&'a mut I2cManager>,
}

impl<'a> PowerManager<'a> {
    pub fn new(i2c: &'a mut I2cManager) -> Result<Self> {
        let mut axp = Axpxx::new(i2c);
        axp.init().map_err(|e| anyhow::anyhow!("{:?}", e))?;
        axp.set_power_output(Power::Ldo2, PowerState::On, &mut FreeRtos)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(Self { axp })
    }

    pub fn read_voltage(&mut self) -> Result<u16> {
        self.axp.get_battery_voltage().map(|v| v as u16).map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}

impl<'a> Drop for PowerManager<'a> {
    fn drop(&mut self) {
        let _ = self.axp.set_power_output(Power::Ldo2, PowerState::Off, &mut FreeRtos);
    }
}