# twatch2020v3_rs
TTGO T-Watch 2020 v3 firmware written in Rustlang

### I/Os
 - Interrupt RTC: IO37
 - PMU/AXP202
   - interrupt IO35
   - I2C_SDA IO21
   - I2C_SCL IO22
 - IR IO13
 - Touch Board
   - interrupt IO38
   - I2C_SDA IO23
   - I2C_SCL IO32
 - PDM_MIC
   - data IO02
   - sclk IO00
 - Vibration Motor: GPIO4
 - LCD 1.54 /ST7789V
   - TFT_MISO NULL
   - TFT_MOSI IO19
   - TFT_SCLK IO18
   - TFT_CS IO05
   - TFT_DC IO27
   - TFT_RST NULL
   - TFT_BL IO15
 - Axis Sensor/BMA423
   - interrupt IO39
   - I2C_SDA IO21
   - I2C_SCL IO22
 - I2S_Class /MAX98357A(Audio)
   - I2S_BCK IO26
   - I2S_WS IO25
   - I2C_DOUT IO33

## References
https://lang-ship.com/blog/work/esp32-std-rust-1/
https://github.com/pyaillet/twatch-idf-rs
