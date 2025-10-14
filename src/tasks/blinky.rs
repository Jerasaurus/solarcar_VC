use defmt::*;
use embassy_stm32::gpio::Output;
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn blinky_task(mut led: Output<'static>) {
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