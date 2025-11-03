/// UDP socket management for vehicle communication
use defmt::*;
use embassy_net::{IpEndpoint, IpListenEndpoint, Stack, udp::{PacketMetadata, UdpSocket}};

use super::config::{
    BROADCAST_ADDRESS, BMS_ADDRESS, BMS_PORT, RECEIVE_PORT,
    TELEMETRY_PORT, VC_ADDRESS, VC_PORT
};

/// Maximum UDP packet size
pub const MAX_PACKET_SIZE: usize = 1024;

/// Send a message to the Vehicle Computer
pub async fn send_to_vc(
    stack: &'static Stack<'static>,
    data: &[u8],
) -> Result<(), embassy_net::udp::SendError> {
    let mut rx_buffer = [0; MAX_PACKET_SIZE];
    let mut tx_buffer = [0; MAX_PACKET_SIZE];
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];

    let mut socket = UdpSocket::new(
        stack.clone(),
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    // Bind to any available port (port 0 means any ephemeral port)
    socket.bind(IpListenEndpoint {
        addr: None,
        port: 0,
    }).ok(); // Ignore error if already bound

    let remote_endpoint = IpEndpoint::new(VC_ADDRESS.into(), VC_PORT);

    debug!("Sending {} bytes to VC at {}", data.len(), remote_endpoint);
    socket.send_to(data, remote_endpoint).await
}

/// Send a message to the Battery Management System
pub async fn send_to_bms(
    stack: &'static Stack<'static>,
    data: &[u8],
) -> Result<(), embassy_net::udp::SendError> {
    let mut rx_buffer = [0; MAX_PACKET_SIZE];
    let mut tx_buffer = [0; MAX_PACKET_SIZE];
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];

    let mut socket = UdpSocket::new(
        stack.clone(),
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    // Bind to any available port (port 0 means any ephemeral port)
    socket.bind(IpListenEndpoint {
        addr: None,
        port: 0,
    }).ok(); // Ignore error if already bound

    let remote_endpoint = IpEndpoint::new(BMS_ADDRESS.into(), BMS_PORT);

    debug!("Sending {} bytes to BMS at {}", data.len(), remote_endpoint);
    socket.send_to(data, remote_endpoint).await
}

/// Broadcast telemetry data
pub async fn broadcast_telemetry(
    stack: &'static Stack<'static>,
    data: &[u8],
) -> Result<(), embassy_net::udp::SendError> {
    let mut rx_buffer = [0; MAX_PACKET_SIZE];
    let mut tx_buffer = [0; MAX_PACKET_SIZE];
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];

    let mut socket = UdpSocket::new(
        stack.clone(),
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    // Bind to any available port (port 0 means any ephemeral port)
    socket.bind(IpListenEndpoint {
        addr: None,
        port: 0,
    }).ok(); // Ignore error if already bound

    let broadcast_endpoint = IpEndpoint::new(BROADCAST_ADDRESS.into(), TELEMETRY_PORT);

    info!("Broadcasting {} bytes of telemetry to {}", data.len(), broadcast_endpoint);
    socket.send_to(data, broadcast_endpoint).await
}

/// Create a UDP socket for receiving messages
pub async fn create_receive_socket<'a>(
    stack: &'static Stack<'static>,
    rx_buffer: &'a mut [u8],
    tx_buffer: &'a mut [u8],
    rx_meta: &'a mut [PacketMetadata],
    tx_meta: &'a mut [PacketMetadata],
) -> Result<UdpSocket<'a>, embassy_net::udp::BindError> {
    let mut socket = UdpSocket::new(
        stack.clone(),
        rx_meta,
        rx_buffer,
        tx_meta,
        tx_buffer,
    );

    let listen_endpoint = IpListenEndpoint {
        addr: None, // Listen on all interfaces
        port: RECEIVE_PORT,
    };

    socket.bind(listen_endpoint)?;
    info!("UDP socket bound to port {}", RECEIVE_PORT);

    Ok(socket)
}