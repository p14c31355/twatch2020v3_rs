// src/drivers/axp.rs
use anyhow::Result;
use axp20x::Axpxx;
use crate::manager::I2cManager;

pub struct PowerManager;

impl PowerManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn get_battery_percentage(&mut self, i2c: &mut I2cManager) -> Result<u8> {
        let mut axp = Axpxx::new(i2c);
        let percentage = axp.get_battery_percentage().map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(percentage)
    }

    pub fn read_voltage(&mut self, i2c: &mut I2cManager) -> Result<u16> {
        let mut axp = Axpxx::new(i2c);
        axp.get_battery_voltage().map(|v| v as u16).map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}