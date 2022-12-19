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
    dma::DmaPriority,
    gdma::Gdma,
    prelude::*,
    spi::{dma::WithDmaSpi2, Spi, SpiMode},
    systimer::SystemTimer,
    timer::TimerGroup,
    Rtc,
    IO,
    Delay,
};

use mipidsi::Orientation;


use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use embedded_graphics::image::Image;
use embedded_graphics::geometry::*;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyleBuilder};
use embedded_graphics_framebuf::FrameBuf;

use profont::{PROFONT_24_POINT, PROFONT_18_POINT};

use core::f32::consts::PI;
use libm::{sin, cos};



#[cfg(feature="xtensa-lx-rt")]
use xtensa_lx_rt::entry;
#[cfg(feature="riscv-rt")]
use riscv_rt::entry;

use esp_println::println;
use esp_backtrace as _;



#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();

    let mut clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

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


    let dma = Gdma::new(peripherals.DMA, &mut system.peripheral_clock_control);
    let dma_channel = dma.channel0;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];
    

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
    let spi = Spi::new(
        peripherals.SPI2,
        sck,
        mosi,
        miso,
        cs,
        100u32.MHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );

    let di = SPIInterfaceNoCS::new(spi, dc.into_push_pull_output());
    let reset = rst.into_push_pull_output();
    let mut delay = Delay::new(&clocks);

    println!("About to initialize display via mipidsi...");

    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(240 as u16, 320 as u16)
        .with_framebuffer_size(240 as u16, 320 as u16)
        .with_orientation(Orientation::Landscape(true))
        .init(&mut delay, Some(reset))
        .unwrap();


    let mut data = [Rgb565::WHITE; 320 * 240];

    let mut fbuf = FrameBuf::new(&mut data, 320, 240);

    #[cfg(feature = "st7789")]
    display.init(&mut delay).unwrap();
    #[cfg(feature = "st7789")]
    display.set_orientation(st7789::Orientation::Portrait).unwrap();
    

    println!("Initialized");

    // display.clear(Rgb565::WHITE).unwrap();

    #[cfg(feature = "st7789")]
    Text::new("Display initialized",
              display.bounding_box().center() - Size::new(display.bounding_box().size.width/2 - 10, 0), 
              MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::BLACK))
    .draw(&mut display)
    .unwrap();
    #[cfg(feature = "ili9341")]
    Text::new("Display initialized",
              display.bounding_box().center() - Size::new(display.bounding_box().size.width/2 - 10, 0), 
              MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::BLACK))
    .draw(&mut fbuf)
    .unwrap();
    display.draw_iter(fbuf.into_iter()).unwrap();

    delay.delay_ms(2000 as u32);

    fbuf.clear(Rgb565::WHITE).unwrap();

    let start_timestamp = SystemTimer::now();

    

    fbuf.clear(Rgb565::WHITE);
    let default_style = MonoTextStyleBuilder::new()
                    .font(&FONT_10X20)
                    .text_color(RgbColor::BLACK)
                    .build();

                let mut n = 6.0;
                let mut d = 71.0;    
                let mut a;
                let mut r;
                let mut x;
                let mut y;

                for t in 0..361 {
                    a = t as f64 * d * (PI as f64 / 60.0);
                    r = 30.0 * sin(n * a);
                    x = r * cos(a);
                    y = r * sin(a);

                    Text::with_alignment("o", Point::new((x + 35.0) as i32, (y + 180.0) as i32), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();
                let pos_x = 1;
                for pos_y in 0..60 {
                    Text::with_alignment("|", Point::new(pos_x + 34, pos_y + 180), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                n = 7.0;
                d = 19.0;
                for t in 0..700 {
                    a = t as f64 * d * (PI as f64 / 300.0);
                    r = 30.0 * sin(n * a);
                    x = r * cos(a);
                    y = r * sin(a);

                    Text::with_alignment("o", Point::new((x + 90.0) as i32, (y + 140.0) as i32), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                for pos_y in 0..100 {
                    Text::with_alignment("|", Point::new(pos_x + 89, pos_y + 140), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                n = 2.0;
                d = 39.0;
                for t in 0..500 {
                    a = t as f64 * d * (PI as f64 / 150.0);
                    r = 30.0 * sin(n * a);
                    x = r * cos(a);
                    y = r * sin(a);

                    Text::with_alignment("S", Point::new((x + 140.0) as i32, (y + 190.0) as i32), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                for pos_y in 0..50 {
                    Text::with_alignment("|", Point::new(pos_x + 139, pos_y + 190), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                n = 8.0;
                d = 27.0;
                for t in 0..1000 {
                    a = t as f64 * d * (PI as f64 / 230.0);
                    r = 30.0 * sin(n * a);
                    x = r * cos(a);
                    y = r * sin(a);

                    Text::with_alignment("o", Point::new((x + 243.0) as i32, (y + 200.0) as i32), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();
                for pos_y in 0..85 {
                    Text::with_alignment("|", Point::new(pos_x + 242, pos_y + 200), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                n = 5.0;
                d = 97.0;
                for t in 0..700 {
                    a = t as f64 * d * (PI as f64 / 150.0);
                    r = 30.0 * sin(n * a);
                    x = r * cos(a);
                    y = r * sin(a);

                    Text::with_alignment("o", Point::new((x + 290.0) as i32, (y + 155.0) as i32), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();
                for pos_y in 0..85 {
                    Text::with_alignment("|", Point::new(pos_x + 289, pos_y + 155), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                n = 6.0;
                d = 71.0;
                for t in 0..2500 {
                    a = t as f64 * d * (PI as f64 / 1200.0);
                    r = 80.0 * sin(n * a);
                    x = r * cos(a);
                    y = r * sin(a);

                    Text::with_alignment("o", Point::new((x + 200.0) as i32, (y + 90.0) as i32), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                for pos_y in 0..100 {
                    Text::with_alignment("|", Point::new(pos_x + 199, pos_y + 140), default_style,  Alignment::Center)
                        .draw(&mut fbuf)
                        .unwrap();
                }
                display.draw_iter(fbuf.into_iter()).unwrap();

                
    let end_timestamp = SystemTimer::now();

    println!("Rendering took : {}ms", (end_timestamp - start_timestamp)/ 100000 );
    loop {}
}