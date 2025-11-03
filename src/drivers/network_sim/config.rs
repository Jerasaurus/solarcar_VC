/// Network configuration (same as real network for compatibility)

/// Static IP configuration for the steering wheel
pub const IP_ADDRESS: [u8; 4] = [192, 168, 0, 30];
pub const NETMASK: [u8; 4] = [255, 255, 255, 0];
pub const GATEWAY: [u8; 4] = [192, 168, 0, 1];

/// Target addresses for communication
pub const VC_ADDRESS: [u8; 4] = [192, 168, 0, 20];
pub const VC_PORT: u16 = 3001;

pub const BMS_ADDRESS: [u8; 4] = [192, 168, 0, 10];
pub const BMS_PORT: u16 = 2001;

/// Local ports
pub const RECEIVE_PORT: u16 = 4001;

/// Telemetry broadcast
pub const BROADCAST_ADDRESS: [u8; 4] = [192, 168, 0, 255];
pub const TELEMETRY_PORT: u16 = 6000;

/// AWS telemetry server
pub const AWS_ADDRESS: [u8; 4] = [3, 149, 38, 188];
pub const AWS_PORT: u16 = 6000;

/// Ethernet hardware address (MAC)
pub const MAC_ADDRESS: [u8; 6] = [0x02, 0x00, 0x11, 0x22, 0x33, 0x44];