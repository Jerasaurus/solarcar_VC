//! USB configuration constants and helpers
//!
//! This module contains all USB-related configuration that can be
//! easily modified for different use cases.

/// USB device information
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: &'static str,
    pub product: &'static str,
    pub serial_number: &'static str,
}

/// Default USB device information for the Vehicle Computer
pub const DEFAULT_USB_INFO: UsbDeviceInfo = UsbDeviceInfo {
    vendor_id: 0x16c0,      // Generic USB vendor ID
    product_id: 0x27dd,     // Generic USB product ID
    manufacturer: "Stanford Solar Car",
    product: "Vehicle Computer Debug",
    serial_number: "001",
};

/// USB buffer sizes
pub mod buffer_sizes {
    /// Size of the USB endpoint buffer in bytes
    pub const ENDPOINT: usize = 256;

    /// Size of the logger buffer in bytes
    pub const LOGGER: usize = 1024;
}

/// USB logging configuration
pub mod logging {
    /// Default log level for USB output
    pub const DEFAULT_LEVEL: log::LevelFilter = log::LevelFilter::Info;

    /// Whether to include timestamps in USB log messages
    pub const INCLUDE_TIMESTAMPS: bool = false;
}