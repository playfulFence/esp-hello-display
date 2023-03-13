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
    clock::{ClockControl, CpuClock},
    peripherals::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Rtc,
    IO,
    Delay,
};

/* Display and graphics */
use mipidsi::{ Orientation, ColorOrder };

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use embedded_graphics::image::Image;
use embedded_graphics::geometry::*;
use embedded_graphics::draw_target::DrawTarget;

use profont::{PROFONT_24_POINT, PROFONT_18_POINT};

use esp_println::println;
use esp_backtrace as _;

#[cfg(any(feature = "esp32c3", feature = "esp32c2"))]
use hal::system::SystemExt;

//static EXECUTOR: StaticCell<Executor> = StaticCell::new();


#[entry]
fn main() -> ! {
    esp_wifi::init_heap();
    let peripherals = Peripherals::take();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();

    #[cfg(feature = "esp32c3")]
    let mut clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
    #[cfg(any(feature = "esp32", feature = "esp32s3", feature = "esp32s2"))]
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;
    
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    #[cfg(not(any(feature = "esp32", feature = "esp32s2")))]
    rtc.swd.disable();

    rtc.rwdt.disable();


    println!("About to initialize the SPI LED driver ILI9341");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    

    /* Set corresponding pins */
    #[cfg(feature = "esp32")]
    let mosi = io.pins.gpio23;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let mosi = io.pins.gpio7;
    #[cfg(feature = "esp32c3")]
    let mosi = io.pins.gpio7;

    #[cfg(feature = "esp32")]
    let cs = io.pins.gpio22;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let cs = io.pins.gpio5;
    #[cfg(feature = "esp32c3")]
    let cs = io.pins.gpio20;

    #[cfg(feature = "esp32")]
    let rst = io.pins.gpio18;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let rst = io.pins.gpio18;
    #[cfg(feature = "esp32c3")]
    let rst = io.pins.gpio3;

    #[cfg(feature = "esp32")]
    let dc = io.pins.gpio21;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let dc = io.pins.gpio4;
    #[cfg(feature = "esp32c3")] 
    let dc = io.pins.gpio21;

    #[cfg(feature = "esp32")]
    let sck = io.pins.gpio19;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let sck = io.pins.gpio6;
    #[cfg(feature = "esp32c3")]
    let sck = io.pins.gpio6;

    #[cfg(feature = "esp32")]
    let miso = io.pins.gpio25;
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let miso = io.pins.gpio12;
    #[cfg(feature = "esp32c3")]
    let miso = io.pins.gpio8;

    #[cfg(feature = "esp32")]
    let mut backlight = io.pins.gpio5.into_push_pull_output();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let mut backlight = io.pins.gpio9.into_push_pull_output();
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
    
    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(240 as u16, 320 as u16)
        .with_framebuffer_size(240 as u16, 320 as u16)
        .with_orientation(Orientation::Landscape((true)))
        .with_color_order(ColorOrder::Bgr)
        .init(&mut delay, Some(reset))
    .unwrap();
    

    println!("Initialized");

    display.clear(Rgb565::WHITE).unwrap();

    // #[cfg(feature = "st7789")]
    // Text::new("Display initialized",
    //           display.bounding_box().center() - Size::new(display.bounding_box().size.width/2 - 10, 0), 
    //           MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::BLACK))
    // .draw(&mut display)
    // .unwrap();
    #[cfg(feature = "ili9341")]
    Text::new("Display initialized",
              display.bounding_box().center() - Size::new(display.bounding_box().size.width/2 - 10, 0), 
              MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::BLACK))
    .draw(&mut display)
    .unwrap();

    loop {}
}
