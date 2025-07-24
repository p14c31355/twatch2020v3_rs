use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, AnyOutputPin, PinDriver}, // AnyIOPin を追加
    peripherals::Peripherals,
    prelude::*,
    spi::{config::{Config as SpiConfig, DriverConfig}, SpiDeviceDriver, SpiDriver},
    sys::EspError,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::{Builder, interface::SpiInterface, models::ST7789};

#[derive(Debug)]
enum Error {
    Esp(EspError),
    Gpio(esp_idf_hal::gpio::GpioError),
    Spi(esp_idf_hal::spi::SpiError),
    MipidsiInit(String),
    Draw(String),
}

impl From<EspError> for Error {
    fn from(e: EspError) -> Self { Error::Esp(e) }
}
impl From<esp_idf_hal::gpio::GpioError> for Error {
    fn from(e: esp_idf_hal::gpio::GpioError) -> Self { Error::Gpio(e) }
}
impl From<esp_idf_hal::spi::SpiError> for Error {
    fn from(e: esp_idf_hal::spi::SpiError) -> Self { Error::Spi(e) }
}

// 適切なバッファサイズを調整してください。240x240ピクセル、Rgb565（2バイト/ピクセル）なら
// 240 * 240 * 2 = 115200バイトが必要です。
// DISPLAY_BUFFERのサイズが足りないと、描画エラーやクラッシュの原因になります。
// `mipidsi`の `SpiInterface::new` は `WriteOnlyDataCommand` を使う場合、
// バッファサイズは転送するデータの最大サイズに合わせる必要があります。
// 全画面描画をバッファで行うなら 115200 を確保すべきですが、
// 通常は描画コマンドの引数や部分描画のために使うので、転送効率が良い範囲で小さくします。
// ここでは、例として一般的な転送バッファサイズとして少し大きめに設定します。
// 実際にフルスクリーン更新を行うにはもっと大きいか、部分的な描画を繰り返すロジックが必要です。
// ここはデモ目的で、一旦大きめに設定しますが、実際にメモリに乗り切らない可能性もあります。
// プロジェクトの必要に応じて調整してください。
static mut DISPLAY_BUFFER: [u8; 16384] = [0u8; 16384]; // 240x240 LCD用ならもっと大きいか、DMAを使うべき。

fn main() -> Result<(), Error> {
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();

    // --- ピン配置の修正 ---
    // T-Watch 2020 V3 ピン配置図に基づく
    let sclk = peripherals.pins.gpio19; // TFT_SCK: IO19
    let mosi = peripherals.pins.gpio23; // TFT_MOSI: IO23
    let miso = None::<AnyIOPin>;       // TFT_MISO: NULL (MISOピンは不要なのでNoneを明示的に型指定)
    let cs: AnyOutputPin = peripherals.pins.gpio5.into(); // TFT_CS: IO5

    let dc = PinDriver::output(peripherals.pins.gpio27)?; // TFT_DC: IO27
    let mut bl = PinDriver::output(peripherals.pins.gpio15)?; // TFT_BL: IO15

    // V3では LOW = ON の可能性あり。とりあえず HIGH から試す
    // バックライトON
    bl.set_low()?; // または bl.set_low()?; どちらか光る方で

    let spi_driver = SpiDriver::new(
        peripherals.spi2, // SPI2を使用
        sclk,
        mosi,
        miso, // <--- ここで型が Option<AnyIOPin> になるように修正済み
        &DriverConfig::new(),
    )?;

    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        Some(cs), // Chip Selectピンは必須
        &SpiConfig::new().baudrate(20.MHz().into()), // 高速化
    )?;

    let mut delay = FreeRtos;

    // `di` は `SpiInterface<SpiDeviceDriver, PinDriver<Output>, &'static mut [u8]>` の型になる
    // DISPLAY_BUFFER は、mipidsi が内部で使う転送バッファです。
    // LCDへのデータ転送時にこのバッファを経由します。
    // フル画面バッファリングとは異なります。
    // 修正後のコード (src/main.rs の 88行目付近)
    let di = unsafe {
    // raw pointer を作成
    let raw_ptr: *mut [u8] = &raw mut DISPLAY_BUFFER;
    // raw pointer から &mut スライスに再変換
    // ここで unsafe が必要なのは、raw pointer から参照を作成するため
    let buffer_slice: &mut [u8] = &mut *raw_ptr;

    SpiInterface::new(spi_device, dc, buffer_slice)
};

    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        // TFT_RSTはNULLなので、reset_pinは指定しない
        // .reset_pin(PinDriver::output(peripherals.pins.gpioXX)?) // 必要であればコメント解除しピンを指定
        .init(&mut delay)
        .map_err(|e| Error::MipidsiInit(format!("{:?}", e)))?;

    display.clear(Rgb565::BLACK)
        .map_err(|e| Error::Draw(format!("{:?}", e)))?;

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    Text::new("Hello TWatch 2020 V3!", Point::new(10, 120), style)
        .draw(&mut display)
        .map_err(|e| Error::Draw(format!("{:?}", e)))?;

    println!("Display initialized and text drawn!");

    loop {
        bl.toggle().expect("Failed to toggle backlight"); // バックライトをトグル
        FreeRtos::delay_ms(1000);
    }
}