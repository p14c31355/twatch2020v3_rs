use esp_idf_hal::gpio::*;
use esp_idf_hal::prelude::*;
use esp_idf_sys::*;
use log::info;
use async_std;
use async_std::channel::{bounded, Sender};
use std::sync::Mutex;
use std::sync::Arc;

#[async_std::main]
async fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = peripherals.pins.gpio35;

    let (tx, rx) = bounded(1); // バッファサイズ1のチャネルを作成
    let tx = Arc::new(Mutex::new(tx)); // Mutexで保護

    unsafe {
        gpio_set_intr_type(pin.pin(), gpio_int_type_t_GPIO_INTR_ANYEDGE); // 割り込みタイプを設定
        gpio_intr_enable(1 << pin.pin()); // 割り込みを有効化
        let tx_clone = Arc::clone(&tx);
        gpio_isr_handler_add(pin.pin(), Some(isr_handler), Arc::into_raw(tx_clone) as *mut std::ffi::c_void); // 割り込みハンドラを設定
    }

    extern "C" fn isr_handler(arg: *mut std::ffi::c_void) {
        let tx = unsafe { Arc::from_raw(arg as *mut Mutex<Sender<bool>>) };
        let state = unsafe { gpio_get_level(35) != 0 }; // GPIO35の状態を取得
        tx.lock().unwrap().try_send(state).unwrap_or_default();
        Arc::into_raw(tx); // Arcを元に戻す
    }

    async_std::task::spawn(async move {
        loop {
            if let Ok(state) = rx.recv().await {
                if state {
                    info!("Button pressed!");
                } else {
                    info!("Button released!");
                }
                async_std::task::sleep(std::time::Duration::from_millis(200)).await; // チャタリング対策
            }
        }
    });

    loop {
        async_std::task::sleep(std::time::Duration::from_millis(1000)).await;
    }
}