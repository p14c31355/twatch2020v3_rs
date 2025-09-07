// src/drivers/axp.rs
use anyhow::Result;
use axp20x::{Axpxx, Power, PowerState};
use crate::manager::I2cManager;
use esp_idf_hal::delay::FreeRtos;

pub struct PowerManager;

impl PowerManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn init_power(&mut self, i2c: &mut I2cManager) -> Result<()> {
        let mut axp = Axpxx::new(i2c);
        axp.init().map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }

    pub fn set_backlight(&mut self, i2c: &mut I2cManager, on: bool) -> Result<()> {
        let mut axp = Axpxx::new(i2c);
        let mut delay = FreeRtos;

        let state = match on {
            true => PowerState::On,
            false => PowerState::Off,
        };

        axp.set_power_output(Power::Ldo2, state, &mut delay)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }

    pub fn get_battery_percentage(&mut self, i2c: &mut I2cManager) -> Result<u8> {
        let mut axp = Axpxx::new(i2c);
        axp.get_battery_percentage()
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }

    pub fn read_voltage(&mut self, i2c: &mut I2cManager) -> Result<u16> {
        let mut axp = Axpxx::new(i2c);
        axp.get_battery_voltage()
            .map(|v| v as u16)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}
