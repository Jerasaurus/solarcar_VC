/// Telemetry broadcast task - sends steering wheel data over UDP
use defmt::*;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};

use crate::drivers::network;

/// Simple test message structure
/// In the future, this will be replaced with protobuf messages
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TelemetryMessage {
    pub sequence: u32,
    pub timestamp: u32,
    pub button_state: u16,
    pub throttle: u16,
    pub brake: u16,
}

impl TelemetryMessage {
    pub fn new() -> Self {
        Self {
            sequence: 0,
            timestamp: 0,
            button_state: 0,
            throttle: 0,
            brake: 0,
        }
    }

    /// Convert to bytes for transmission
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.sequence.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.button_state.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.throttle.to_le_bytes());
        bytes[12..14].copy_from_slice(&self.brake.to_le_bytes());
        // Last 2 bytes are padding
        bytes
    }
}

/// Telemetry broadcast task
///
/// Sends telemetry data every second to:
/// - Broadcast address (192.168.0.255:6000)
/// - AWS telemetry server (if configured)
#[embassy_executor::task]
pub async fn telemetry_task(stack: &'static Stack<'static>) {
    info!("Starting telemetry broadcast task");

    // Wait for network to be ready
    network::wait_for_link_up(stack).await;

    let mut sequence = 0u32;
    let mut message = TelemetryMessage::new();

    loop {
        // Update message with current data
        message.sequence = sequence;
        message.timestamp = embassy_time::Instant::now().as_millis() as u32;

        // TODO: Get actual button and pedal states from shared state
        // For now, use test values
        message.button_state = (sequence & 0xFF) as u16; // Test pattern
        message.throttle = ((sequence * 100) % 4096) as u16; // Simulate ADC value
        message.brake = ((sequence * 50) % 4096) as u16;

        // Convert to bytes
        let data = message.to_bytes();

        // Broadcast telemetry
        match network::broadcast_telemetry(stack, &data).await {
            Ok(()) => {
                info!("Telemetry broadcast #{} sent successfully", sequence);
            }
            Err(e) => {
                error!("Failed to broadcast telemetry: {:?}", e);
            }
        }

        sequence = sequence.wrapping_add(1);

        // Wait 1 second before next broadcast
        Timer::after(Duration::from_secs(1)).await;
    }
}

/// Steering wheel update task (50ms cycle)
///
/// This will eventually:
/// - Read button states
/// - Read ADC values for pedals
/// - Send updates to VC and BMS
#[embassy_executor::task]
pub async fn steering_update_task(stack: &'static Stack<'static>) {
    info!("Starting steering wheel update task");

    // Wait for network to be ready
    network::wait_for_link_up(stack).await;

    let mut sequence = 0u32;

    loop {
        // Create test message
        let mut message = TelemetryMessage::new();
        message.sequence = sequence;
        message.timestamp = embassy_time::Instant::now().as_millis() as u32;

        // TODO: Get actual states
        message.button_state = 0x0001; // Test: first button pressed
        message.throttle = 2048; // Test: 50% throttle
        message.brake = 0;

        let data = message.to_bytes();

        // Send to Vehicle Computer
        match network::send_to_vc(stack, &data).await {
            Ok(()) => {
                debug!("Update #{} sent to VC", sequence);
            }
            Err(e) => {
                error!("Failed to send to VC: {:?}", e);
            }
        }

        // Send to BMS
        match network::send_to_bms(stack, &data).await {
            Ok(()) => {
                debug!("Update #{} sent to BMS", sequence);
            }
            Err(e) => {
                error!("Failed to send to BMS: {:?}", e);
            }
        }

        sequence = sequence.wrapping_add(1);

        // Wait 50ms for next update
        Timer::after(Duration::from_millis(50)).await;
    }
}