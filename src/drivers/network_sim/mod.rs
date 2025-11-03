/// Simulated network module for testing without real Ethernet hardware
///
/// This module provides the same interface as the real network module
/// but simulates UDP communication for testing purposes.
/// Replace with real Ethernet implementation when hardware support is available.

use defmt::*;
use embassy_time::{Duration, Timer};
use heapless::Vec;

pub mod config;
pub use config::*;

/// Simulated network stack
pub struct SimulatedStack {
    ip: [u8; 4],
}

impl SimulatedStack {
    pub fn new() -> Self {
        Self {
            ip: [192, 168, 0, 30],
        }
    }

    /// Wait for simulated link to be up
    pub async fn wait_config_up(&self) {
        info!("Simulated network: waiting for link...");
        Timer::after(Duration::from_millis(100)).await;
        info!("Simulated network: link up at {}.{}.{}.{}",
              self.ip[0], self.ip[1], self.ip[2], self.ip[3]);
    }
}

/// Initialize simulated network
pub fn init_network() -> &'static SimulatedStack {
    static STACK: SimulatedStack = SimulatedStack {
        ip: [192, 168, 0, 30],
    };

    info!("Initialized simulated network stack");
    &STACK
}

/// Wait for network to be ready
pub async fn wait_for_link_up(stack: &'static SimulatedStack) {
    stack.wait_config_up().await;
}

/// Send a message to the Vehicle Computer (simulated)
pub async fn send_to_vc(
    _stack: &'static SimulatedStack,
    data: &[u8],
) -> Result<(), &'static str> {
    debug!("SIM: Sending {} bytes to VC at 192.168.0.20:3001", data.len());

    // Simulate network delay
    Timer::after(Duration::from_micros(100)).await;

    // Log first few bytes for debugging
    if data.len() >= 4 {
        debug!("SIM: Data[0..4] = {:02x} {:02x} {:02x} {:02x}",
               data[0], data[1], data[2], data[3]);
    }

    Ok(())
}

/// Send a message to the Battery Management System (simulated)
pub async fn send_to_bms(
    _stack: &'static SimulatedStack,
    data: &[u8],
) -> Result<(), &'static str> {
    debug!("SIM: Sending {} bytes to BMS at 192.168.0.10:2001", data.len());

    // Simulate network delay
    Timer::after(Duration::from_micros(100)).await;

    Ok(())
}

/// Broadcast telemetry data (simulated)
pub async fn broadcast_telemetry(
    _stack: &'static SimulatedStack,
    data: &[u8],
) -> Result<(), &'static str> {
    info!("SIM: Broadcasting {} bytes of telemetry", data.len());

    // Simulate network delay
    Timer::after(Duration::from_micros(200)).await;

    // Log the telemetry data
    if data.len() >= 16 {
        let sequence = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let timestamp = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        info!("SIM: Telemetry seq={} time={}", sequence, timestamp);
    }

    Ok(())
}

/// Simulated receive buffer
pub struct SimulatedReceiver {
    counter: u32,
}

impl SimulatedReceiver {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Simulate receiving a message
    pub async fn receive(&mut self) -> Vec<u8, 256> {
        // Wait some time to simulate network timing
        Timer::after(Duration::from_secs(2)).await;

        // Create a fake message
        let mut msg = Vec::new();

        // Add some test data
        msg.extend_from_slice(&self.counter.to_le_bytes()).ok();
        msg.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]).ok();

        self.counter += 1;

        info!("SIM: Received message #{}", self.counter);

        msg
    }
}