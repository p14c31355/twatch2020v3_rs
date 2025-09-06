use anyhow::Result;
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::delay::FreeRtos;
use axp20x::{Axpxx, Power, PowerState};

pub struct PowerManager<'a> {
    pub axp: Axpxx<&'a mut I2cDriver<'a>>,
}

impl<'a> PowerManager<'a> {
    pub fn new(i2c: &'a mut I2cDriver<'a>) -> Result<Self> {
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
