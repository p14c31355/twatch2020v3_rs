use anyhow::Result;
use embedded_hal::blocking::i2c::{Write, WriteRead};

pub struct Axp192<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Axp192<I2C>
where
    I2C: Write + WriteRead,
{
    /// AXP192インスタンス生成
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// 任意レジスタ読み出し
    pub fn read_reg(&mut self, reg: u8) -> Result<u8> {
        let mut buf = [0u8];
        self.i2c.write_read(self.address, &[reg], &mut buf)?;
        Ok(buf[0])
    }

    /// 任意レジスタ書き込み
    pub fn write_reg(&mut self, reg: u8, value: u8) -> Result<()> {
        self.i2c.write(self.address, &[reg, value])?;
        Ok(())
    }

    /// 指定ビットセット（ビットマスク）
    pub fn set_bits(&mut self, reg: u8, mask: u8) -> Result<()> {
        let val = self.read_reg(reg)?;
        self.write_reg(reg, val | mask)
    }

    /// 指定ビットクリア（ビットマスク）
    pub fn clear_bits(&mut self, reg: u8, mask: u8) -> Result<()> {
        let val = self.read_reg(reg)?;
        self.write_reg(reg, val & !mask)
    }

    /// IRQ許可（ビットマスク指定）
    pub fn enable_irq(&mut self, mask: u8) -> Result<()> {
        const IRQ_EN_REG: u8 = 0x46;
        self.set_bits(IRQ_EN_REG, mask)
    }

    /// IRQ無効化（ビットマスク指定）
    pub fn disable_irq(&mut self, mask: u8) -> Result<()> {
        const IRQ_EN_REG: u8 = 0x46;
        self.clear_bits(IRQ_EN_REG, mask)
    }

    /// IRQステータス読み出し
    pub fn read_irq_status(&mut self) -> Result<u8> {
        const IRQ_STATUS_REG: u8 = 0x48;
        self.read_reg(IRQ_STATUS_REG)
    }

    /// IRQステータスクリア
    pub fn clear_irq_status(&mut self, mask: u8) -> Result<()> {
        const IRQ_STATUS_REG: u8 = 0x48;
        self.write_reg(IRQ_STATUS_REG, mask)
    }

    /// バックライト制御（例：バックライトのON/OFF切り替え）
    pub fn set_backlight(&mut self, enable: bool) -> Result<()> {
        const BACKLIGHT_REG: u8 = 0x12; // 仮のレジスタ番号、要実機確認
        if enable {
            self.set_bits(BACKLIGHT_REG, 0x04) // 仮のマスク
        } else {
            self.clear_bits(BACKLIGHT_REG, 0x04)
        }
    }
}
