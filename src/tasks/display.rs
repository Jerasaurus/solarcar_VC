use defmt::*;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_time::Timer;
use crate::drivers::display::Ssd1322Display;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Gray4,
    prelude::*,
    text::{Alignment, Text},
};

#[embassy_executor::task]
pub async fn display_task(
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