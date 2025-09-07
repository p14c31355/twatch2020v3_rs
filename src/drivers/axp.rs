use anyhow::Result;
use axp20x::{Axpxx, PowerOutput};
use crate::manager::I2cManager;

pub struct PowerManager;

impl PowerManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn init_power(&mut self, i2c: &mut I2cManager) -> Result<()> {
        let mut axp = Axpxx::new(i2c);

        axp.set_power_output(PowerOutput::LDO2, true, 0)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        axp.set_power_output(PowerOutput::LDO3, true, 0)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        axp.set_power_output(PowerOutput::DCDC1, true, 0)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        axp.set_charge_enable(true)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(())
    }

    pub fn get_battery_percentage(&mut self, i2c: &mut I2cManager) -> Result<u8> {
        let mut axp = Axpxx::new(i2c);
        let percentage = axp
            .get_battery_percentage()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(percentage)
    }

    pub fn read_voltage(&mut self, i2c: &mut I2cManager) -> Result<u16> {
        let mut axp = Axpxx::new(i2c);
        axp.get_battery_voltage()
            .map(|v| v as u16)
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }
}
