pub type EspSharedBusI2c0<'a> = shared_bus::I2cProxy<'a, std::sync::Mutex<EspI2c0>>;

use crate::gpio::Output;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::{gpio, prelude::*};
use log::*;
use std::time;
use esp_idf_hal::spi;
use mipidsi::*;

pub type EspI2c0 = esp_idf_hal::i2c<
    gpio::Gpio35<gpio::Output>,
    gpio::Gpio21<gpio::Output>,
    gpio::Gpio22<gpio::Output>,
>;

pub type EspSpi2Interface = SPIInterface<
    spi::Master<
        spi::SPI3,
        gpio::Gpio18<Output>,
        gpio::Gpio19<Output>,
        gpio::Gpio23<Output>,
        gpio::Gpio5<Output>,
    >,
    gpio::Gpio27<Output>,
>;

pub struct TwatchDisplay {
    pub display: Display<EspSpi2InterfaceNoCS, mipidsi::NoPin, mipidsi::models::ST7789>,
    pub backlight: Backlight,
    pub framebuffer: &'static mut FrameBuf<Rgb565, 57600_usize>
}

impl DrawTarget for TwatchDisplay {
    type Color = Rgb565;

    type Error = TwatchError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.framebuffer
            //self.display
            .draw_iter(pixels)
            .map_err(|_| TwatchError::Display)
    }
}

impl OriginDimensions for TwatchDisplay {
    fn size(&self) -> Size {
        Size {
            width: 240,
            height: 240,
        }
    }
}

pub struct Backlight {
    channel: Channel<CHANNEL0, TIMER0, Arc<Timer<TIMER0>>, Gpio12<Output>>,
    level: u32,
}

impl Backlight {
    pub fn new(channel: CHANNEL0, timer: TIMER0, backlight: Gpio12<Output>) -> Self {
        let config = TimerConfig::default().frequency(5.kHz().into());
        let timer0 =
            Arc::new(Timer::new(timer, &config).expect("Unable to create timer for backlight"));
        let channel = Channel::new(channel, timer0, backlight)
            .expect("Unable to create channel for backlight");
        Self {
            channel,
            level: 100,
        }
    }
}

impl TwatchDisplay {
    pub fn new(di: EspSpi2InterfaceNoCS, backlight: Backlight) -> Result<Self> {
        let display = Display::st7789_without_rst(di);
        static mut FBUFF: FrameBuf<Rgb565, 240_usize, 240_usize, 57_600_usize> =
            FrameBuf([Rgb565::BLACK; 57_600]);
        let framebuffer = unsafe { &mut FBUFF };

        Ok(Self {
            display,
            backlight,
            framebuffer,
        })
    }
    pub fn init(&mut self, delay_source: &mut impl DelayUs<u32>) -> Result<()> {
        let display_options = DisplayOptions {
            color_order: ColorOrder::Bgr,
            ..Default::default()
        };
        self.display
            .init(delay_source, display_options)
            .map_err(|e| {
                info!("Error initializing display {e:?}");
                TwatchError::Display
            })?;
        Ok(())
    }

    pub fn commit_display_partial(&mut self, rect: Rectangle) -> Result<()> {
        if rect.size == self.bounding_box().size {
            self.display
                .write_raw(
                    rect.top_left.x as u16,
                    rect.top_left.y as u16,
                    rect.top_left.x as u16 + rect.size.width as u16,
                    rect.top_left.y as u16 + rect.size.height as u16,
                    self.framebuffer.as_words(),
                )
                .map_err(|_| TwatchError::Display)?;
        } else {
            let mut partial_fb: Vec<u16> = vec![0; (rect.size.width * rect.size.height) as usize];
            let sx: usize = rect.top_left.x as _;
            let ex: usize = (rect.size.width + rect.top_left.x as u32) as _;

            for i in rect.rows() {
                let dsx = (rect.size.width * i as u32) as _;
                let dex = (rect.size.width * (i + 1 as u32) as u32) as _;
                let ssx = sx + (i as u32 * 240) as usize;
                let sex = ex + (i as u32 * 240) as usize;
                partial_fb[dsx..dex].copy_from_slice(&self.framebuffer.as_words()[ssx..sex]);
            }

            self.display
                .write_raw(
                    rect.top_left.x as u16,
                    rect.top_left.y as u16,
                    rect.top_left.x as u16 + rect.size.width as u16 - 1,
                    rect.top_left.y as u16 + rect.size.height as u16 - 1,
                    &mut partial_fb,
                )
                .map_err(|_| TwatchError::Display)?;
        }
        Ok(())
    }

    pub fn commit_display(&mut self) -> Result<()> {
        self.commit_display_partial(Rectangle {
            top_left: Point::default(),
            size: Size {
                width: 240,
                height: 240,
            },
        })?;

        self.framebuffer.clear_black();
        Ok(())
    }

    pub fn get_display_level(&self) -> u32 {
        self.backlight.level
    }

    pub fn set_display_level<I: Into<u32>>(&mut self, level: I) -> Result<()> {
        self.backlight.level = level.into();
        let max_duty = self.backlight.channel.get_max_duty();
        self.backlight
            .channel
            .set_duty(self.backlight.level * max_duty / 100)?;
        Ok(())
    }

    pub fn set_display_on(&mut self) -> Result<()> {
        self.set_display_level(100u32)?;
        Ok(())
    }

    pub fn set_display_off(&mut self) -> Result<()> {
        self.set_display_level(0u32)?;
        Ok(())
    }
}

/*
pub struct Hal {
    pub motor: gpio::Gpio4<Output>, // define Vibration motor
}

pub struct Pmu<'a> {
    axp20x: axp20x::Axpxx<EspSharedBusI2c0<'a>>,
}
*/

impl Twatch {
    pub fn new(peripherals: Peripherals) -> Self {
        let pins = peripherals.pins;
        /*
        let motor = pins
            .gpio4
            .into_output()
            .expect("Unable to set gpio4 to output");
        */
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum State {
    On,
    Off,
}
/*
impl From<State> for axp20x::PowerState {
    fn from(state: State) -> Self {
        match state {
            State::On => axp20x::PowerState::On,
            State::Off => axp20x::PowerState::Off,
        }
    }
}
*/

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

/*
pub fn button_to_motor(&mut self) -> Result<()> {
    Ok(())
}
*/
