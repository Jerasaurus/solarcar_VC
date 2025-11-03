/// Ethernet PHY initialization and management with LAN8742A support
use defmt::*;
use embassy_net::{Stack, StackResources};
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::ETH;
use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, eth, peripherals, rng, Peri};
use embassy_time::Timer;
use static_cell::StaticCell;

use super::config::{MAC_ADDRESS, NETWORK_CONFIG, GATEWAY};

// Bind the ETH interrupt
bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});

// Device type alias
pub type Device = Ethernet<'static, ETH, GenericPhy>;

// Static storage for the network stack
static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

/// Network task that runs the network stack
#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, Device>) -> ! {
    runner.run().await
}

/// Reset the LAN8742A PHY chip (blocking version)
///
/// This is a blocking version that should be called before initializing Ethernet
/// to avoid async task conflicts during startup.
pub fn reset_phy_blocking(p_pd15: Peri<'static, peripherals::PD15>) {
    info!("Resetting LAN8742A PHY (blocking)...");

    let mut nreset = Output::new(p_pd15, Level::High, Speed::Low);

    // Initial high state
    nreset.set_high();
    cortex_m::asm::delay(168_000 * 2); // ~2ms at 168MHz

    // Assert reset (active low)
    nreset.set_low();
    cortex_m::asm::delay(168_000 * 2); // ~2ms at 168MHz

    // De-assert reset
    nreset.set_high();

    // Wait for PHY to stabilize after reset
    cortex_m::asm::delay(168_000 * 100); // ~100ms at 168MHz

    info!("PHY reset complete");
}

/// Reset the LAN8742A PHY chip (async version)
///
/// This performs the reset sequence for the LAN8742A PHY:
/// 1. Set nRESET high (inactive)
/// 2. Wait 2ms
/// 3. Pull nRESET low (active reset)
/// 4. Wait 2ms
/// 5. Release nRESET high (inactive)
/// 6. Wait for PHY to stabilize
pub async fn reset_phy(p_pd15: Peri<'static, peripherals::PD15>) {
    info!("Resetting LAN8742A PHY...");

    let mut nreset = Output::new(p_pd15, Level::High, Speed::Low);

    // Initial high state
    nreset.set_high();
    Timer::after_millis(2).await;

    // Assert reset (active low)
    nreset.set_low();
    Timer::after_millis(2).await;

    // De-assert reset
    nreset.set_high();

    // Wait for PHY to stabilize after reset
    Timer::after_millis(100).await;

    info!("PHY reset complete");
}

/// Initialize the Ethernet hardware and network stack
///
/// This configures the STM32F4's Ethernet MAC with RMII interface
/// and sets up the embassy-net stack with a static IP configuration.
///
/// Returns (stack, runner) - the runner must be spawned as a task
pub fn init_ethernet(
    p_eth: Peri<'static, peripherals::ETH>,
    p_pa1: Peri<'static, peripherals::PA1>,
    p_pa2: Peri<'static, peripherals::PA2>,
    p_pa7: Peri<'static, peripherals::PA7>,
    p_pb11: Peri<'static, peripherals::PB11>,
    p_pb12: Peri<'static, peripherals::PB12>,
    p_pb13: Peri<'static, peripherals::PB13>,
    p_pc1: Peri<'static, peripherals::PC1>,
    p_pc4: Peri<'static, peripherals::PC4>,
    p_pc5: Peri<'static, peripherals::PC5>,
    p_rng: Peri<'static, peripherals::RNG>,
    seed: u64,
) -> (&'static Stack<'static>, embassy_net::Runner<'static, Device>) {
    info!("Initializing Ethernet hardware...");

    // Create packet queue (required for Ethernet driver)
    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();

    // Create Ethernet device
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        p_eth,
        Irqs,
        p_pa1,   // REF_CLK
        p_pa2,   // MDIO
        p_pc1,   // MDC
        p_pa7,   // CRS_DV
        p_pc4,   // RXD0
        p_pc5,   // RXD1
        p_pb12,  // TXD0
        p_pb13,  // TXD1
        p_pb11,  // TX_EN
        GenericPhy::new_auto(), // Auto-detect PHY at address 0
        MAC_ADDRESS,
    );

    // Initialize random number generator for network protocols
    let _rng = Rng::new(p_rng, Irqs);

    // Configure the network stack with static IP
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: NETWORK_CONFIG,
        gateway: Some(GATEWAY),
        dns_servers: Default::default(),
    });

    // Initialize the network stack
    let (stack, runner) = embassy_net::new(
        device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    info!("Network stack initialized with IP: {}", NETWORK_CONFIG.address());

    // Need to store stack in static storage and return reference
    static STACK: StaticCell<Stack<'static>> = StaticCell::new();
    let stack = STACK.init(stack);

    (stack, runner)
}

/// Wait for the network link to be up
pub async fn wait_for_link_up(stack: &'static Stack<'static>) {
    info!("Waiting for network configuration...");
    stack.wait_config_up().await;
    info!("Network configuration ready!");

    // Wait a bit for link to stabilize
    embassy_time::Timer::after_millis(500).await;

    info!("Network ready!");
}