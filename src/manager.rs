use esp_idf_hal::i2c::{I2cDriver, I2cError};
use heapless::{Vec, consts::U256, FnvIndexMap};
use crossbeam_channel::Sender;

pub enum I2cRequest<const N: usize> {
    Write {
        addr: u8,
        data: Vec<u8, N>,
        resp: Sender<Result<(), I2cError>>,
    },
    Read {
        addr: u8,
        len: usize,
        resp: Sender<Result<Vec<u8, N>, I2cError>>,
    },
}

pub struct I2cManager {
    tx: Sender<I2cRequest<U256>>,
}

impl I2cManager {
    pub fn new(mut i2c: I2cDriver<'static>) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();

        const BLOCK: Option<core::time::Duration> = Some(core::time::Duration::from_millis(100));

        std::thread::spawn(move || {
            while let Ok(req) = rx.recv() {
                match req {
                    I2cRequest::Write { addr, data, resp } => {
                        let res = i2c.write(addr, &data, BLOCK)
                                     .map_err(|e| e);
                        let _ = resp.send(res);
                    }
                    I2cRequest::Read { addr, len, resp } => {
                        let mut buf: Vec<u8, N> = Vec::new();
                        buf.resize_default(len).ok();
                        let res = i2c.read(addr, &mut buf, BLOCK)
                                     .map(|_| buf)
                                     .map_err(|e| e);
                        let _ = resp.send(res);
                    }
                }
            }
        });

        I2cManager { tx }
    }

    pub fn write(&self, addr: u8, data: &[u8]) -> Result<(), I2cError> { // Changed N to U256
        let mut buf: Vec<u8, U256> = Vec::new();
        buf.extend_from_slice(data).map_err(|_| I2cError::Other)?; // Use a specific I2cError variant
        let (resp_tx, resp_rx) = crossbeam_channel::bounded(1);
        self.tx.send(I2cRequest::Write { addr, data: buf, resp: resp_tx }).unwrap();
        resp_rx.recv().unwrap()
    }

    pub fn read(&self, addr: u8, len: usize) -> Result<Vec<u8, U256>, I2cError> { // Changed N to U256
        let (resp_tx, resp_rx) = crossbeam_channel::bounded(1);
        self.tx.send(I2cRequest::Read { addr, len, resp: resp_tx }).unwrap();
        resp_rx.recv().unwrap()
    }
}
