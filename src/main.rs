#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::Timer;
use embassy_vehiclecomputer::drivers::display::Ssd1322Display;
use embassy_vehiclecomputer::usb::setup_usb_logger;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Gray4,
    prelude::*,
    text::{Alignment, Text},
};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Configure clocks using 25 MHz external oscillator
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,  // 25MHz / 25 = 1MHz
            mul: PllMul::MUL336,       // 1MHz * 336 = 336MHz VCO
            divp: Some(PllPDiv::DIV2), // 336MHz / 2 = 168MHz system clock
            divq: Some(PllQDiv::DIV7), // 336MHz / 7 = 48MHz USB clock
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4; // 168/4 = 42 MHz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 168/2 = 84 MHz
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
    }

    let p = embassy_stm32::init(config);

    // Initialize LED
    let led = Output::new(p.PD8, Level::Low, Speed::Low);

    // Initialize USB logger
    setup_usb_logger(&spawner, p.USB_OTG_FS, p.PA12, p.PA11).unwrap();

    // Configure SPI1 for display
    let mut spi_config = spi::Config::default();
    spi_config.mode = spi::Mode {
        polarity: spi::Polarity::IdleHigh,
        phase: spi::Phase::CaptureOnSecondTransition,
    };
    spi_config.frequency = Hertz(10_000_000); // 10 MHz

    let spi = Spi::new(
        p.SPI1,
        p.PB3,       // SCLK
        p.PB5,       // MOSI
        p.PB4,       // MISO (not used but required)
        p.DMA2_CH3,  // TX DMA
        p.DMA2_CH0,  // RX DMA
        spi_config,
    );

    // Configure display control pins
    let dc = Output::new(p.PB6, Level::High, Speed::High);   // Data/Command
    let cs = Output::new(p.PA15, Level::High, Speed::High);  // Chip Select
    let rst = Output::new(p.PD7, Level::High, Speed::High);  // Reset

    // Spawn tasks
    spawner.spawn(display_task(spi, dc, cs, rst)).unwrap();
    spawner.spawn(blinky_task(led)).unwrap();
}

#[embassy_executor::task]
async fn blinky_task(mut led: Output<'static>) {
    info!("Blinky started!");
    Timer::after_millis(1000).await;
    log::info!("USB Logger: Blinky task started on PD8");

    let mut counter = 0u32;
    loop {
        led.set_high();
        info!("LED ON - count {}", counter);
        log::info!("USB: LED ON - count {}", counter);
        Timer::after_millis(1000).await;

        led.set_low();
        info!("LED OFF - count {}", counter);
        log::info!("USB: LED OFF - count {}", counter);
        Timer::after_millis(1000).await;

        counter += 1;
    }
}

#[embassy_executor::task]
async fn display_task(
    spi: Spi<'static, Async>,
    dc: Output<'static>,
    cs: Output<'static>,
    rst: Output<'static>,
) {
    info!("Display task started!");

    // Initialize display
    let mut display = Ssd1322Display::new(spi, dc, cs, rst).await;
    Timer::after_millis(100).await;
    log::info!("USB: Display initialized");

    // Animation variables
    let mut brightness = 0u8;
    let mut increasing = true;
    let mut y_offset = 0i32;

    loop {
        // Clear display
        display.clear();

        // Create dynamic style based on current brightness
        let text_style = MonoTextStyle::new(&FONT_10X20, Gray4::new(brightness));

        // Draw "HELLO WORLD" centered with current brightness
        Text::with_alignment(
            "HELLO WORLD",
            Point::new(128, 32 + y_offset), // Center of 256x64 display
            text_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .ok();

        // Update display
        display.flush().await;

        // Animate brightness
        if increasing {
            if brightness < 15 {
                brightness += 1;
            } else {
                increasing = false;
            }
        } else {
            if brightness > 1 {
                brightness -= 1;
            } else {
                increasing = true;
                // Small vertical animation when cycle completes
                y_offset = (y_offset + 1) % 5 - 2; // Oscillate between -2 and 2
            }
        }

        Timer::after_millis(100).await;
    }
}