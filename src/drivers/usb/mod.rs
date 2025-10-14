//! USB communication module for the Vehicle Computer
//!
//! This module provides USB functionality for debugging and communication.
//! Currently supports:
//! - USB serial logging for debug messages
//!
//! # Module Structure
//!
//! - `config` - Configuration constants and defaults for USB operation
//! - `setup` - USB initialization and setup functions
//!
//! # Usage
//!
//! ```no_run
//! use embassy_vehiclecomputer::usb;
//!
//! // In your main function:
//! usb::init_logger(&spawner, usb_peripheral, dp_pin, dm_pin)?;
//! ```

pub mod config;
mod setup;

// Re-export the main USB initialization function with a simpler name
pub use setup::setup_usb_logger as init_logger;

// Re-export for backwards compatibility
pub use setup::setup_usb_logger;