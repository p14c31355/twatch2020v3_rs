// src/drivers/axp.rs
use crate::manager::I2cManager;
use anyhow::Result;
use axp20x::{Axpxx, Power, PowerState};
use esp_idf_hal::delay::FreeRtos;

pub struct PowerManager {
    axp: Axpxx<I2cManager>,
}

impl PowerManager {
    pub fn new(i2c: I2cManager) -> Result<Self> {
        let mut axp = Axpxx::new(i2c);
        axp.init().map_err(|e| anyhow::anyhow!("AXP init failed: {:?}", e))?;
        Ok(Self { axp })
    }

    pub fn set_backlight(&mut self, on: bool) -> Result<()> {
        let mut delay = FreeRtos;
        let state = if on { PowerState::On } else { PowerState::Off };
        self.axp
            .set_power_output(Power::Ldo2, state, &mut delay)
            .map_err(|e| anyhow::anyhow!("Failed to set backlight: {:?}", e))?;
        Ok(())
    }

    pub fn get_battery_percentage(&mut self) -> Result<u8> {
        self.axp
            .get_battery_percentage()
            .map_err(|e| anyhow::anyhow!("Failed to read battery percentage: {:?}", e))
    }

    pub fn read_voltage(&mut self) -> Result<u16> {
        self.axp
            .get_battery_voltage()
            .map(|v| v as u16)
            .map_err(|e| anyhow::anyhow!("Failed to read battery voltage: {:?}", e))
    }

    pub fn set_power(&mut self, channel: Power, state: PowerState) -> Result<()> {
        let mut delay = FreeRtos;
        self.axp
            .set_power_output(channel, state, &mut delay)
            .map_err(|e| anyhow::anyhow!("Failed to set power channel: {:?}", e))
    }
}
