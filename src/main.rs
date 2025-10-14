#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_vehiclecomputer::drivers::buttons::{ButtonInputs, Button, ButtonId};
use embassy_vehiclecomputer::drivers::usb::setup_usb_logger;
use embassy_vehiclecomputer::tasks;
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

    // Initialize LED on PD8 (starts OFF)
    let led = Output::new(p.PD8, Level::Low, Speed::Low);

    // Initialize USB logger for debugging
    // This creates a USB serial device that will appear on your computer
    // You can connect to it with a serial terminal to see log messages
    // USB pins: PA12 (D+) and PA11 (D-)
    setup_usb_logger(&spawner, p.USB_OTG_FS, p.PA12, p.PA11)
        .expect("Failed to initialize USB logger");

    // Initialize button inputs - all button definitions in one place!
    // To add a new button:
    // 1. Add its ButtonId variant to the enum in drivers/buttons/mod.rs
    // 2. Add a Button entry here with the pin assignment
    let button_inputs = ButtonInputs::new([
        Button::regular(ButtonId::CruiseDown,  "Cruise Down",   p.PD12),
        Button::regular(ButtonId::CruiseUp,    "Cruise Up",     p.PE14),
        Button::regular(ButtonId::Reverse,     "Reverse",       p.PE0),
        Button::regular(ButtonId::PushToTalk,  "Push-to-Talk",  p.PE4),
        Button::regular(ButtonId::Horn,        "Horn",          p.PD14),
        Button::regular(ButtonId::PowerSave,   "Power Save",    p.PE2),
        Button::regular(ButtonId::Rearview,    "Rearview",      p.PE8),
        Button::toggle(ButtonId::LeftTurn,     "Left Turn",     p.PE12),
        Button::toggle(ButtonId::RightTurn,    "Right Turn",    p.PE6),
        Button::toggle(ButtonId::Lock,         "Lock",          p.PE10),
    ]);

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
    spawner.spawn(tasks::display_task(spi, dc, cs, rst)).unwrap();
    spawner.spawn(tasks::blinky_task(led)).unwrap();
    spawner.spawn(tasks::button_task(button_inputs)).unwrap();
}