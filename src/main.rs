#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Configure clocks using 25 MHz external oscillator
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        // 25 MHz external oscillator
        config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::Oscillator,  // Not bypass mode - actual oscillator
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,  // 25MHz / 25 = 1MHz
            mul: PllMul::MUL336,       // 1MHz * 336 = 336MHz VCO
            divp: Some(PllPDiv::DIV2), // 336MHz / 2 = 168MHz system clock
            divq: Some(PllQDiv::DIV7), // 336MHz / 7 = 48MHz USB clock!
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;  // 168/4 = 42 MHz (max 45)
        config.rcc.apb2_pre = APBPrescaler::DIV2;  // 168/2 = 84 MHz (max 90)
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
    }

    let p = embassy_stm32::init(config);

    // Initialize LED
    let led = Output::new(p.PD8, Level::Low, Speed::Low);

    // Create USB driver for logger
    static EP_OUT_BUFFER: static_cell::StaticCell<[u8; 256]> = static_cell::StaticCell::new();
    let ep_out_buffer = EP_OUT_BUFFER.init([0u8; 256]);

    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;

    let driver = embassy_stm32::usb::Driver::new_fs(p.USB_OTG_FS, Irqs, p.PA12, p.PA11, ep_out_buffer, usb_config);

    // Spawn USB logger task
    spawner.spawn(logger_task(driver)).unwrap();

    // Spawn blinky task
    spawner.spawn(blinky_task(led)).unwrap();
}

#[embassy_executor::task]
async fn logger_task(driver: embassy_stm32::usb::Driver<'static, peripherals::USB_OTG_FS>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn blinky_task(mut led: Output<'static>) {
    info!("Blinky started!");

    // Wait a bit for USB to enumerate
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