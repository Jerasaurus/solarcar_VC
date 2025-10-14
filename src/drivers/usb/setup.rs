//! USB configuration and setup for debugging over USB serial
//!
//! This module provides a simple USB logger setup that allows you to view
//! debug messages over a USB serial connection.

use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, peripherals, usb, Peri};
use embassy_stm32::usb::Driver;

use super::config;

// ============================================================================
// USB Interrupt Handler
// ============================================================================

// USB interrupt handler binding for STM32F4's USB OTG Full-Speed peripheral
bind_interrupts!(pub struct UsbIrqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

// ============================================================================
// Public API
// ============================================================================

/// Initialize and start the USB logger for debugging
///
/// This sets up a USB serial device that can be used to view log messages
/// from a host computer. Once connected, all `log::info!()`, `log::warn!()`,
/// etc. messages will be sent over USB.
///
/// # Arguments
/// * `spawner` - Embassy task spawner for running the USB logger task
/// * `usb_peripheral` - The USB OTG Full-Speed peripheral
/// * `usb_dp` - USB D+ pin (PA12 on most STM32F4 boards)
/// * `usb_dm` - USB D- pin (PA11 on most STM32F4 boards)
///
/// # Returns
/// * `Ok(())` if the logger task was spawned successfully
/// * `Err(SpawnError)` if the task could not be spawned
///
/// # Example
/// ```no_run
/// let p = embassy_stm32::init(config);
/// setup_usb_logger(&spawner, p.USB_OTG_FS, p.PA12, p.PA11)?;
/// ```
pub fn setup_usb_logger(
    spawner: &Spawner,
    usb_peripheral: Peri<'static, peripherals::USB_OTG_FS>,
    usb_dp: Peri<'static, peripherals::PA12>,  // D+ pin
    usb_dm: Peri<'static, peripherals::PA11>,  // D- pin
) -> Result<(), embassy_executor::SpawnError> {
    // Create a static buffer for USB endpoint operations
    // This buffer is used for USB data transfers
    static EP_OUT_BUFFER: static_cell::StaticCell<[u8; config::buffer_sizes::ENDPOINT]> =
        static_cell::StaticCell::new();
    let ep_out_buffer = EP_OUT_BUFFER.init([0u8; config::buffer_sizes::ENDPOINT]);

    // Configure USB settings
    let mut usb_config = embassy_stm32::usb::Config::default();
    // Disable VBUS detection since we're always USB-powered
    usb_config.vbus_detection = false;

    // Create the USB driver for Full-Speed operation (12 Mbps)
    let driver = Driver::new_fs(
        usb_peripheral,
        UsbIrqs,
        usb_dp,
        usb_dm,
        ep_out_buffer,
        usb_config,
    );

    // Spawn the logger task to handle USB communication
    spawner.spawn(usb_logger_task(driver))
}

// ============================================================================
// Internal Tasks
// ============================================================================

/// Embassy task that runs the USB logger
///
/// This task continuously handles USB communication and forwards log messages
/// to the host computer via USB serial.
#[embassy_executor::task]
async fn usb_logger_task(driver: Driver<'static, peripherals::USB_OTG_FS>) {
    // Start the USB logger with configured buffer size and log level
    embassy_usb_logger::run!(
        { config::buffer_sizes::LOGGER },
        config::logging::DEFAULT_LEVEL,
        driver
    );
}