use defmt::*;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_time::{Duration, Instant, Timer};
use crate::drivers::display::Ssd1322Display;
use crate::drivers::display::DriveState;
use crate::drivers::display::ssd1322::DISPLAY_BLACK;

// Display state structure
struct DisplayState {
    current_screen: u8,
    left_blink: bool,
    right_blink: bool,
    last_blink: u32,
    bms_flash: bool,
    last_flash: u32,
}

impl DisplayState {
    fn new() -> Self {
        Self {
            current_screen: 0,
            left_blink: false,
            right_blink: false,
            last_blink: 0,
            bms_flash: false,
            last_flash: 0,
        }
    }
}

// Placeholder vehicle state - in production this would come from CAN/network messages
struct VehicleState {
    drive_mode: DriveState,
    left_motor_velocity: f32,
    right_motor_velocity: f32,
    cruise_enabled: bool,
    cruise_speed: f32,
    regen_enabled: bool,
    brake_pressed: bool,
    throttle_enabled: bool,
    throttle_pressed: bool,
    battery_current: f32,
    high_voltage: f32,
    low_voltage: f32,
    lock_on: bool,
    bps_strobe: bool,
    throttle_value: f32,
    raw_throttle: u16,
    regen_value: f32,
    raw_regen: u32,
    pedal_value: f32,
    raw_pedal: u32,
}

impl Default for VehicleState {
    fn default() -> Self {
        Self {
            drive_mode: DriveState::Neutral,
            left_motor_velocity: 0.0,
            right_motor_velocity: 0.0,
            cruise_enabled: false,
            cruise_speed: 0.0,
            regen_enabled: true,
            brake_pressed: false,
            throttle_enabled: true,
            throttle_pressed: false,
            battery_current: 0.0,
            high_voltage: 120.0,
            low_voltage: 12.5,
            lock_on: false,
            bps_strobe: false,
            throttle_value: 0.0,
            raw_throttle: 0,
            regen_value: 0.0,
            raw_regen: 0,
            pedal_value: 0.0,
            raw_pedal: 0,
        }
    }
}

#[embassy_executor::task]
pub async fn display_task(
    spi: Spi<'static, Async>,
    dc: Output<'static>,
    cs: Output<'static>,
    rst: Output<'static>,
) {
    info!("Display task started!");

    // Initialize display
    let mut display = Ssd1322Display::new(spi, dc, cs, rst).await;
    Timer::after_millis(100).await;
    info!("Display initialized");

    let mut state = DisplayState::new();
    let vehicle_state = VehicleState::default();
    
    // Timing variables
    let start_time = Instant::now();
    let mut time_since_vc = 0u32;
    let mut time_since_bms = 0u32;

    loop {
        let current_time = start_time.elapsed().as_millis() as u32;
        
        // TODO: Update vehicle_state from actual CAN messages or network data
        // For now using placeholder values
        
        // Update time since last message
        time_since_vc += 50; // Placeholder - would be updated when actual message received
        time_since_bms += 50; // Placeholder - would be updated when actual message received
        
        // Clear display
        display.fill(DISPLAY_BLACK);

        match state.current_screen {
            0 => {
                // Main screen
                display.write_drive_state(vehicle_state.drive_mode);
                
                let max_velocity = vehicle_state.left_motor_velocity.max(vehicle_state.right_motor_velocity);
                display.write_speed(max_velocity);
                
                display.write_turn_signal_state(
                    &mut state.left_blink,
                    &mut state.right_blink,
                    &mut state.last_blink,
                    current_time,
                );
                
                display.write_cruise_speed(vehicle_state.cruise_enabled, vehicle_state.cruise_speed);
                
                display.write_regen(
                    vehicle_state.regen_enabled,
                    vehicle_state.brake_pressed,
                    vehicle_state.throttle_value > 0.2,
                );
                
                display.write_throttle(
                    vehicle_state.throttle_enabled,
                    vehicle_state.throttle_pressed,
                    vehicle_state.throttle_value > 0.2,
                );
                
                display.write_current(vehicle_state.battery_current);
                display.write_high_voltage(vehicle_state.high_voltage);
                display.write_low_voltage(vehicle_state.low_voltage);
                display.write_lock(vehicle_state.lock_on);
                
                display.write_bms_flash(
                    vehicle_state.bps_strobe,
                    &mut state.bms_flash,
                    &mut state.last_flash,
                    current_time,
                );
                
                display.write_timeout(time_since_vc);
                display.write_bms_timeout(time_since_bms);
            }
            1 => {
                // Debug screen
                display.write_timeout(time_since_vc);
                display.write_bms_timeout(time_since_bms);
                
                display.write_throttle_debug(vehicle_state.throttle_value, vehicle_state.raw_throttle);
                display.write_regen_debug(vehicle_state.regen_value, vehicle_state.raw_regen);
                display.write_pedal_value(vehicle_state.pedal_value, vehicle_state.raw_pedal);
                
                display.write_debug();
            }
            _ => {
                // Unknown screen, default to main
                state.current_screen = 0;
            }
        }

        // Flush display
        display.flush().await;

        // Run at ~20Hz (50ms period) to match the C implementation
        Timer::after(Duration::from_millis(10)).await;
    }
}