use anyhow::Result;
use embedded_hal::i2c::I2c;

pub struct Axp192<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Axp192<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    pub fn read_reg(&mut self, reg: u8) -> Result<u8> {
        let mut buf = [0u8];
        self.i2c.write_read(self.address, &[reg], &mut buf)?;
        Ok(buf[0])
    }

    pub fn write_reg(&mut self, reg: u8, value: u8) -> Result<()> {
        self.i2c.write(self.address, &[reg, value])?;
        Ok(())
    }

    pub fn set_bits(&mut self, reg: u8, mask: u8) -> Result<()> {
        let val = self.read_reg(reg)?;
        self.write_reg(reg, val | mask)
    }

    pub fn clear_bits(&mut self, reg: u8, mask: u8) -> Result<()> {
        let val = self.read_reg(reg)?;
        self.write_reg(reg, val & !mask)
    }

    pub fn enable_irq(&mut self, mask: u8) -> Result<()> {
        const IRQ_EN_REG: u8 = 0x46;
        self.set_bits(IRQ_EN_REG, mask)
    }

    pub fn disable_irq(&mut self, mask: u8) -> Result<()> {
        const IRQ_EN_REG: u8 = 0x46;
        self.clear_bits(IRQ_EN_REG, mask)
    }

    pub fn read_irq_status(&mut self) -> Result<u8> {
        const IRQ_STATUS_REG: u8 = 0x48;
        self.read_reg(IRQ_STATUS_REG)
    }

    pub fn clear_irq_status(&mut self, mask: u8) -> Result<()> {
        const IRQ_STATUS_REG: u8 = 0x48;
        self.write_reg(IRQ_STATUS_REG, mask)
    }
}
