// src/manager.rs
use embedded_hal::i2c::{ErrorType, Operation, SevenBitAddress};
use esp_idf_hal::i2c::{I2cDriver, I2cError};
use esp_idf_sys as _; // for esp-idf FFI
use std::sync::Arc;

const TIMEOUT_MS: u64 = 100;

// FreeRTOS Mutex wrapper
pub struct EspMutex<T> {
    inner: Arc<spin::Mutex<T>>, // spin::Mutex is non-blocking -> WDT safety
}

impl<T> EspMutex<T> {
    pub fn new(t: T) -> Self {
        EspMutex {
            inner: Arc::new(spin::Mutex::new(t)),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<'_, T> {
        self.inner.lock()
    }
}

#[derive(Clone)]
pub struct I2cManager {
    i2c: Arc<EspMutex<I2cDriver<'static>>>,
}

impl I2cManager {
    pub fn new(i2c: I2cDriver<'static>) -> Self {
        Self {
            i2c: Arc::new(EspMutex::new(i2c)),
        }
    }

    fn timeout_ms() -> u32 {
        TIMEOUT_MS.try_into().unwrap_or(u32::MAX)
    }
}

impl ErrorType for I2cManager {
    type Error = I2cError;
}

impl embedded_hal::i2c::I2c<SevenBitAddress> for I2cManager {
    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let timeout = Self::timeout_ms();
        self.i2c.lock().read(addr, buffer, timeout).map_err(Into::into)
    }

    fn write(&mut self, addr: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        let timeout = Self::timeout_ms();
        self.i2c.lock().write(addr, buffer, timeout).map_err(Into::into)
    }

    fn write_read(&mut self, addr: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
        let timeout = Self::timeout_ms();
        self.i2c.lock().write_read(addr, wr_buffer, rd_buffer, timeout).map_err(Into::into)
    }

    fn transaction(&mut self, addr: u8, operations: &mut [Operation]) -> Result<(), Self::Error> {
        let timeout = Self::timeout_ms();
        self.i2c.lock().transaction(addr, operations, timeout).map_err(Into::into)
    }
}
