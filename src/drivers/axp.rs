use anyhow::Result;
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::delay::FreeRtos;
use axp20x::{Axpxx, Power, PowerState};

pub struct PowerManager {
    pub axp: Axpxx<&'static mut I2cDriver<'static>>,
}

impl PowerManager {
    pub fn new(i2c: &'static mut I2cDriver<'static>) -> Result<Self> {
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
}
