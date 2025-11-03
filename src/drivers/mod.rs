pub mod buttons;
pub mod display;
pub mod network;  // Real network with LAN8742A PHY
// pub mod network_sim;  // Simulated network for testing
pub mod usb;

// Use real network implementation
// pub use network_sim as network;