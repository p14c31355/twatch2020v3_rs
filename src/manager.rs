// src/manager.rs
use esp_idf_hal::i2c::{I2cDriver, I2cError, I2c};
use esp_idf_sys::EspError;
use heapless::{Vec, consts::U256};
use crossbeam_channel::Sender;
use std::thread;
use std::sync::Arc;
use core::time::Duration;
use embedded_hal::i2c::{SevenBitAddress, Operation, ErrorKind, ErrorType};

#[derive(Debug)]
pub enum I2cRequest {
    Write {
        addr: u8,
        data: Vec<u8, U256>,
        resp: Sender<Result<(), EspError>>,
    },
    Read {
        addr: u8,
        len: usize,
        resp: Sender<Result<Vec<u8, U256>, EspError>>,
    },
    Transactions {
        addr: u8,
        operations: Vec<Operation<'static>, U256>,
        resp: Sender<Result<(), EspError>>,
    },
}

#[derive(Clone)]
pub struct I2cManager {
    tx: Sender<I2cRequest>,
}

impl I2cManager {
    pub fn new(mut i2c: I2cDriver<'static>) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        thread::spawn(move || {
            while let Ok(req) = rx.recv() {
                match req {
                    I2cRequest::Write { addr, data, resp } => {
                        let res = i2c.write(addr, &data, Duration::from_millis(100).as_millis() as u32);
                        let _ = resp.send(res);
                    }
                    I2cRequest::Read { addr, len, resp } => {
                        let mut buf: Vec<u8, U256> = Vec::new();
                        if buf.resize_default(len).is_ok() {
                            let res = i2c.read(addr, &mut buf, Duration::from_millis(100).as_millis() as u32)
                                .map(|_| buf);
                            let _ = resp.send(res);
                        }
                    }
                    _ => {}
                }
            }
        });

        I2cManager { tx }
    }
}

impl embedded_hal::i2c::Error for I2cError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            I2cError::EspError(_) => ErrorKind::Other,
            _ => ErrorKind::Other,
        }
    }
}

impl ErrorType for &I2cManager {
    type Error = I2cError;
}

impl embedded_hal::i2c::I2c<SevenBitAddress> for &I2cManager {
    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let (resp_tx, resp_rx) = crossbeam_channel::bounded(1);
        self.tx.send(I2cRequest::Read { addr, len: buffer.len(), resp: resp_tx }).unwrap();
        let received_data = resp_rx.recv().unwrap().map_err(|e| I2cError::EspError(e))?;
        buffer.copy_from_slice(&received_data[..]);
        Ok(())
    }

    fn write(&mut self, addr: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        let mut buf: Vec<u8, U256> = Vec::new();
        buf.extend_from_slice(buffer).map_err(|_| I2cError::EspError(EspError::from(0)))?; // Placeholder error
        let (resp_tx, resp_rx) = crossbeam_channel::bounded(1);
        self.tx.send(I2cRequest::Write { addr, data: buf, resp: resp_tx }).unwrap();
        resp_rx.recv().unwrap().map_err(|e| I2cError::EspError(e))
    }

    fn write_read(&mut self, addr: u8, wr_buffer: &[u8], rd_buffer: &mut [u8]) -> Result<(), Self::Error> {
        let mut ops: Vec<Operation, U256> = Vec::new();
        ops.push(Operation::Write(wr_buffer)).map_err(|_| I2cError::EspError(EspError::from(0)))?;
        ops.push(Operation::Read(rd_buffer)).map_err(|_| I2cError::EspError(EspError::from(0)))?;
        let (resp_tx, resp_rx) = crossbeam_channel::bounded(1);
        self.tx.send(I2cRequest::Transactions { addr, operations: ops, resp: resp_tx }).unwrap();
        resp_rx.recv().unwrap().map_err(|e| I2cError::EspError(e))
    }

    fn transaction(&mut self, addr: u8, operations: &mut [Operation]) -> Result<(), Self::Error> {
        let mut ops: Vec<Operation, U256> = Vec::new();
        for op in operations {
            ops.push(op.clone()).map_err(|_| I2cError::EspError(EspError::from(0)))?;
        }
        let (resp_tx, resp_rx) = crossbeam_channel::bounded(1);
        self.tx.send(I2cRequest::Transactions { addr, operations: ops, resp: resp_tx }).unwrap();
        resp_rx.recv().unwrap().map_err(|e| I2cError::EspError(e))
    }
}