use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, peripherals, usb, Peri};
use embassy_stm32::usb::Driver;

bind_interrupts!(pub struct UsbIrqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

pub fn setup_usb_logger(
    spawner: &Spawner,
    usb_peripheral: Peri<'static, peripherals::USB_OTG_FS>,
    pa12: Peri<'static, peripherals::PA12>,
    pa11: Peri<'static, peripherals::PA11>,
) -> Result<(), embassy_executor::SpawnError> {
    // Initialize USB logger
    static EP_OUT_BUFFER: static_cell::StaticCell<[u8; 256]> = static_cell::StaticCell::new();
    let ep_out_buffer = EP_OUT_BUFFER.init([0u8; 256]);

    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;

    let driver = Driver::new_fs(
        usb_peripheral,
        UsbIrqs,
        pa12,
        pa11,
        ep_out_buffer,
        usb_config,
    );

    spawner.spawn(logger_task(driver))
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, peripherals::USB_OTG_FS>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}