#![no_std]
#![no_main]

use esp32s2_hal::{
    clock::ClockControl,
    pac::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Rtc,
    IO,
    Delay,
    systimer::{SystemTimer},
};

/* Display and graphics */
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use embedded_graphics::image::Image;
use embedded_graphics::geometry::*;

use profont::{PROFONT_24_POINT};
use embedded_graphics::draw_target::DrawTarget;

use esp_backtrace as _;
use xtensa_atomic_emulation_trap as _;
use xtensa_lx_rt::entry;
use esp_println::println;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
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

    println!("About to initialize the SPI LED driver ILI9341");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    /* Set corresponding pins */
    let mosi = io.pins.gpio35;
    let cs = io.pins.gpio15;
    let rst = io.pins.gpio4;
    let dc = io.pins.gpio2;
    let sck = io.pins.gpio36;
    let miso = io.pins.gpio10;
    let backlight = io.pins.gpio6;

    /* Then set backlight (set_low() - display lights up when signal is in 0, set_high() - opposite case(for example.)) */
    let mut backlight = backlight.into_push_pull_output();
    //backlight.set_high().unwrap();

    /* Configure SPI */
    let spi = spi::Spi::new(
        peripherals.SPI2,
        sck,
        mosi,
        miso,
        cs,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );


    let di = SPIInterfaceNoCS::new(spi, dc.into_push_pull_output());
    let reset = rst.into_push_pull_output();
    let mut delay = Delay::new(&clocks);
    
    let mut display = Ili9341::new(
        di,
        reset, 
        &mut delay, 
        KalugaOrientation::LandscapeFlipped, 
        DisplaySize240x320
        ).unwrap();
        
    println!("Initialized");

    display.clear(Rgb565::WHITE).unwrap();

    Text::new("Display initialized",
              display.bounding_box().center() - Size::new(display.bounding_box().size.width/2 - 10, 0), 
              MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::BLACK))
    .draw(&mut display)
    .unwrap();

    loop {}
}