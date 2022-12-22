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
use embedded_hal;


use profont::{PROFONT_24_POINT, PROFONT_18_POINT};

use riscv_rt;
use riscv_rt::entry;

use esp_println::println;
use esp_backtrace as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
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
    let mosi = io.pins.gpio7;
    let cs = io.pins.gpio2;
    let rst = io.pins.gpio10;
    let dc = io.pins.gpio3;
    let sck = io.pins.gpio6;
    let miso = io.pins.gpio8;
    let backlight = io.pins.gpio4;

    /* Then set backlight (set_low() - display lights up when signal is in 0, set_high() - opposite case(for example.)) */
    let mut backlight = backlight.into_push_pull_output();
    //backlight.set_low().unwrap();


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
    
    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(240 as u16, 320 as u16)
        .with_framebuffer_size(240 as u16, 320 as u16)
        .with_orientation(Orientation::LandscapeInverted(true))
        .with_color_order(ColorOrder::Bgr)
        .init(&mut delay, Some(reset))
    .unwrap();
        


    println!("Initialized");

    display.clear(Rgb565::WHITE).unwrap();

    Text::new("Display initialized",
              display.bounding_box().center() - Size::new(display.bounding_box().size.width/2 - 10, 0), 
              MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::BLACK))
    .draw(&mut display)
    .unwrap();

    loop {}
}