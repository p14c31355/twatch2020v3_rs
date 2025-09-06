# twatch2020v3-rs
TTGO T-Watch 2020 v3 firmware written in Rustlang

### I/Os
 - Interrupt RTC/PCF8563: IO37
 - PMU/AXP202(Button-pull up resistance)
   - interrupt IO35
   - I2C_SDA IO21
   - I2C_SCL IO22
 - IR IO13
 - Touch Board/FT6236
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

### Install guide ( Linux )
1.espup install
```
cargo install ldproxy
cargo install espup --locked
espup install
source $HOME/export-esp.sh
cat $HOME/export-esp.sh >> ~/.bashrc
```
2.cargo-generate install (option)
```
cargo install cargo-generate
cargo generate esp-rs/esp-idf-template cargo
```
3.build (!!! ESSENTIAL !!!)
```
cargo +esp build
```
!!! "+esp" is ESSENTIAL !!!

### Install guide ( Windows )
1. Install wsl2 & Install WSL extention in your VSCode
2. Activate wsl2 & Restart your PC
3. Install Rust on wsl2
4. Install build-essential on wsl2
5. espup install
