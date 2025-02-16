use std::time;

fn main() {
    // 今の時刻を取得
    let now = time::Instant::now();

    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");
    // 最初の時刻からの経過時間を表示
    println!("{:?}", now.elapsed());
}
