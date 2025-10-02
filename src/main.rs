#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Use default configuration
    let config = Config::default();
    
    let p = embassy_stm32::init(config);
    
    // Try common LED pins - PC13 is common on many STM32 boards
    let mut led = Output::new(p.PD8, Level::Low, Speed::Low);

    loop {
        led.set_high();
        Timer::after_millis(1000).await;
        
        led.set_low();
        Timer::after_millis(1000).await;
    }
}