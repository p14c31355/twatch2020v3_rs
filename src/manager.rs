// src/manager.rs
use esp_idf_hal::i2c::{I2cDriver, I2cError};
use embedded_hal::i2c::{SevenBitAddress, Operation, ErrorType};
use core::time::Duration;
const TIMEOUT_MS: u64 = 10;

pub struct I2cManager {
    i2c: I2cDriver<'static>,
}

impl I2cManager {
    pub fn new(i2c: I2cDriver<'static>) -> Self {
        Self { i2c }
    }
}

impl ErrorType for I2cManager {
    type Error = I2cError;
}

impl embedded_hal::i2c::I2c<SevenBitAddress> for &mut I2cManager {
    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        Ok(self.i2c.read(addr, buffer, Duration::from_millis(TIMEOUT_MS).as_millis() as u32)?)
    }

    fn write(&mut self, addr: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        Ok(self.i2c.write(addr, buffer, Duration::from_millis(TIMEOUT_MS).as_millis() as u32)?)
    }

    fn write_read(&mut self, addr: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
        Ok(self.i2c.write_read(addr, wr_buffer, rd_buffer, Duration::from_millis(TIMEOUT_MS).as_millis() as u32)?)
    }

    fn transaction(&mut self, addr: u8, operations: &mut [Operation]) -> Result<(), Self::Error> {
        Ok(self.i2c.transaction(addr, operations, Duration::from_millis(TIMEOUT_MS).as_millis() as u32)?)
    }
}