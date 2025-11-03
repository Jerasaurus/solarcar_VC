/// Network configuration for the steering wheel system
use embassy_net::{Ipv4Address, Ipv4Cidr};

/// Static IP configuration for the steering wheel
pub const IP_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 0, 30);
pub const NETMASK: Ipv4Address = Ipv4Address::new(255, 255, 255, 0);
pub const GATEWAY: Ipv4Address = Ipv4Address::new(192, 168, 0, 1);

/// Network configuration
pub const NETWORK_CONFIG: Ipv4Cidr = Ipv4Cidr::new(IP_ADDRESS, 24);

/// Target addresses for communication
pub const VC_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 0, 20);
pub const VC_PORT: u16 = 3001;

pub const BMS_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 0, 10);
pub const BMS_PORT: u16 = 2001;

/// Local ports
pub const RECEIVE_PORT: u16 = 4001;

/// Telemetry broadcast
pub const BROADCAST_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 0, 255);
pub const TELEMETRY_PORT: u16 = 6000;

/// AWS telemetry server
pub const AWS_ADDRESS: Ipv4Address = Ipv4Address::new(3, 149, 38, 188);
pub const AWS_PORT: u16 = 6000;

/// Ethernet hardware address (MAC)
/// You can generate a random MAC or use a fixed one
/// Format: 02:xx:xx:xx:xx:xx (locally administered)
pub const MAC_ADDRESS: [u8; 6] = [0x02, 0x00, 0x11, 0x22, 0x33, 0x44];