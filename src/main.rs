#![no_std]
#![no_main]

#[cfg(feature="esp32")]
use esp32_hal as hal;
#[cfg(feature="esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature="esp32s3")]
use esp32s3_hal as hal;
#[cfg(feature="esp32c3")]
use esp32c3_hal as hal;

use hal::{
    clock::ClockControl,
    pac::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Rtc,
    IO,
    Delay,
};

/* Display and graphics */
#[cfg(feature = "ili9341")]
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use embedded_graphics::image::Image;
use embedded_graphics::geometry::*;
use embedded_graphics::draw_target::DrawTarget;

use profont::{PROFONT_24_POINT};



#[cfg(feature="xtensa-lx-rt")]
use xtensa_lx_rt::entry;
#[cfg(feature="riscv-rt")]
use riscv_rt::entry;

use esp_println::println;
use esp_backtrace as _;


/* Some stuff for correct orientation and color on ILI9341 */
pub enum KalugaOrientation {
    Portrait,
    PortraitFlipped,
    Landscape,
    LandscapeVericallyFlipped,
    LandscapeFlipped,
}

impl ili9341::Mode for KalugaOrientation {
    fn mode(&self) -> u8 {
        match self {
            Self::Portrait => 0,
            Self::LandscapeVericallyFlipped => 0x20,
            Self::Landscape => 0x20 | 0x40,
            Self::PortraitFlipped => 0x80 | 0x40,
            Self::LandscapeFlipped => 0x80 | 0x20 | 0x08,
        }
    }

    fn is_landscape(&self) -> bool {
        matches!(self, Self::Landscape | Self::LandscapeFlipped | Self::LandscapeVericallyFlipped)
    }
}


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();

    let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;
    
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();


    println!("About to initialize the SPI LED driver ILI9341");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    

    /* Set corresponding pins */
    #[cfg(feature = "esp32")]
    let mosi = io.pins.gpio23;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let mosi = io.pins.gpio21;
    #[cfg(feature = "esp32c3")]
    let mosi = io.pins.gpio7;

    #[cfg(feature = "esp32")]
    let cs = io.pins.gpio15;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let cs = io.pins.gpio9;
    #[cfg(feature = "esp32c3")]
    let cs = io.pins.gpio20;

    #[cfg(feature = "esp32")]
    let rst = io.pins.gpio4;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let rst = io.pins.gpio0;
    #[cfg(feature = "esp32c3")]
    let rst = io.pins.gpio3;

    #[cfg(feature = "esp32")]
    let dc = io.pins.gpio2;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let dc = io.pins.gpio1;
    #[cfg(feature = "esp32c3")]
    let dc = io.pins.gpio21;

    #[cfg(feature = "esp32")]
    let sck = io.pins.gpio18;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let sck = io.pins.gpio8;
    #[cfg(feature = "esp32c3")]
    let sck = io.pins.gpio6;

    #[cfg(feature = "esp32")]
    let miso = io.pins.gpio8;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let miso = io.pins.gpio4;
    #[cfg(feature = "esp32c3")]
    let miso = io.pins.gpio8;

    #[cfg(feature = "esp32")]
    let mut backlight = io.pins.gpio17.into_push_pull_output();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let mut backlight = io.pins.gpio20.into_push_pull_output();
    #[cfg(feature = "esp32c3")]
    let mut backlight = io.pins.gpio0.into_push_pull_output();

    /* Then set backlight (set_low() - display lights up when signal is in 0, set_high() - opposite case(for example.)) */
    let mut backlight = backlight.into_push_pull_output();
    backlight.set_low().unwrap();


    /* Configure SPI */
    #[cfg(feature = "esp32")]
    let spi = spi::Spi::new(
        peripherals.SPI3,
        sck,
        mosi,
        miso,
        cs,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let spi = spi::Spi::new(
        peripherals.SPI2,
        sck,
        mosi,
        miso,
        cs,
        80u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );
    #[cfg(feature = "esp32c3")]
    let spi = spi::Spi::new(
        peripherals.SPI2,
        sck,
        mosi,
        miso,
        cs,
        80u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );

    let di = SPIInterfaceNoCS::new(spi, dc.into_push_pull_output());
    let reset = rst.into_push_pull_output();
    let mut delay = Delay::new(&clocks);
    let mut display = Ili9341::new(di, reset, &mut delay, KalugaOrientation::LandscapeFlipped, DisplaySize240x320).unwrap();

    println!("Initialized");

    display.clear(Rgb565::WHITE).unwrap();

    Text::new("Display initialized",
              display.bounding_box().center() - Size::new(display.bounding_box().size.width/2 - 10, 0), 
              MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::BLACK))
    .draw(&mut display)
    .unwrap();

    loop {}
}