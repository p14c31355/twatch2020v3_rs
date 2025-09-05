use esp_idf_hal::adc::oneshot::AdcDriver;
use esp_idf_hal::adc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::adc::config::Config;

/// VBATをADCで読み取り、電圧に換算して返す
pub fn read_battery_voltage() -> f32 {
    // Peripherals取得
    let peripherals = Peripherals::take().unwrap();

    // ADC1 初期化
    // AdcDriver を使用し、ADC1 ペリフェラルとデフォルト設定で初期化
    let mut adc1 = AdcDriver::new(peripherals.adc1, Config::new()).unwrap();

    // VBAT接続ピンを選択 (例: GPIO35)
    // GPIO35 を ADC1 のチャンネルとして使用
    let vbat_pin: AnyIOPin = peripherals.pins.gpio35.into(); // GPIO35をVBATピンとして仮定
    let mut vbat_channel = AdcChannelDriver::new(vbat_pin, &mut adc1).unwrap();

    // ADC読み取り
    let reading: u16 = adc1.read(&mut vbat_channel).unwrap_or(0);

    // 分圧補正と電圧換算
    // 例: 分圧1:1なら *2、ADCフルスケール 12bit (4095)、基準電圧 3.3V
    reading as f32 * 3.3 / 4095.0 * 2.0
}
