/// Network communication module for Ethernet and UDP
pub mod config;
pub mod ethernet;
pub mod udp;

pub use config::*;
pub use ethernet::{init_ethernet, wait_for_link_up, net_task, reset_phy, reset_phy_blocking, Device};
pub use udp::*;