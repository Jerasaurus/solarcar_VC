# Steering Wheel System Migration Plan: C to Rust

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [C Codebase Analysis](#c-codebase-analysis)
3. [Current Rust Implementation Status](#current-rust-implementation-status)
4. [Migration Requirements](#migration-requirements)
5. [Proposed Rust Architecture](#proposed-rust-architecture)
6. [Implementation Plan](#implementation-plan)
7. [Technical Details](#technical-details)

---

## Executive Summary

This document outlines the migration plan for the Stanford Solar Car steering wheel system from C (STM32F427) to Rust (STM32F429VI). The C codebase is a complete, production-ready system with ~3,500 lines of code running on FreeRTOS. The Rust codebase currently has ~832 lines with basic infrastructure (Embassy async runtime, display driver, button handling) but lacks vehicle-specific functionality.

**Target Microcontroller:** STM32F429VI (168 MHz, Cortex-M4)
**Runtime:** Embassy (async Rust embedded framework)
**Network Stack:** embassy-net with smoltcp
**Key Peripherals:** 10 buttons, 14 LEDs, OLED display (256x64), 2 ADC channels, Ethernet

---

## C Codebase Analysis

### System Overview
The steering wheel embedded firmware is a comprehensive vehicle control interface that:
- Manages 10 button inputs with debouncing and toggle logic
- Controls 14 LEDs (10 button LEDs + 4 status LEDs)
- Displays telemetry on a 256x64 OLED (SSD1322)
- Reads throttle/brake pedals via ADC
- Communicates via Ethernet UDP with Vehicle Computer and Battery Management System
- Uses protobuf for message serialization

### Hardware Configuration

#### Buttons (10 total)
```c
Pin buttons[STEER_NUMBUTTONS] = {
    [BUT_CRUISE_DOWN]   = {GPIOD, 12},
    [BUT_CRUISE_UP]     = {GPIOE, 14},
    [BUT_Reverse]       = {GPIOE, 0},
    [BUT_Push_To_Talk]  = {GPIOE, 4},
    [BUT_Horn]          = {GPIOD, 14},
    [BUT_PowerSave]     = {GPIOE, 2},
    [BUT_Rearview]      = {GPIOE, 8},
    [BUT_LeftTurn]      = {GPIOE, 12},  // Toggle mode
    [BUT_RightTurn]     = {GPIOE, 6},   // Toggle mode
    [BUT_Lock]          = {GPIOE, 10},  // Toggle mode
};
```

#### LEDs
- 10 button LEDs on various GPIO pins (active low)
- 4 system status LEDs on PD8-PD11 (white, blue, green, red)

#### ADC Channels
- Throttle: PB1 (ADC channel 9)
- Brake: PB0 (ADC channel 8)
- 12-bit resolution, 84-cycle sample time
- Low-pass IIR filter with α=0.5

#### Display
- SSD1322 OLED: 256x64 pixels, 16 grayscale levels
- SPI1 interface at 22 MHz
- Control pins: CS (PA15), DC (PB6), Reset (PD7)

#### Ethernet
- IP: 192.168.0.30
- RMII interface with specific pin mappings
- PHY reset on PD15

### Communication Protocol

#### UDP Messages
1. **Steering → VC** (192.168.0.20:3001) @ 50ms intervals
2. **Steering → BMS** (192.168.0.10:2001) @ 50ms intervals
3. **Receive** on port 4001
4. **Telemetry broadcast** (192.168.0.255:6000) @ 1000ms intervals

#### Message Format (Protobuf)
```proto
message DataMessage {
    SW_State sw_state;    // Steering wheel state
    VC_State vc_state;    // Vehicle computer state
    BMS_State bms_state;  // Battery management state
}
```

### FreeRTOS Tasks

| Task | Priority | Stack | Frequency | Function |
|------|----------|-------|-----------|----------|
| Steering Update | 2 | 3000 | 50ms | Read buttons/ADC, send UDP |
| Display | 0 | 700 | 33ms | Update OLED display |
| LED Update | 0 | 150 | Continuous | Control LEDs |
| Network Receive | 1 | 3500 | Event-driven | Receive UDP messages |
| Telemetry Send | 2 | 3500 | 1000ms | Broadcast telemetry |

### Key Algorithms

#### Debouncing
- 10ms timer per button
- Validates state persistence through debounce window
- Toggle buttons use XOR logic for state changes

#### Pedal Processing
- Dynamic calibration via PTT button press
- Dead zones and linear interpolation
- Returns normalized 0-1 values

#### Display Features
- Main screen: Speed, battery info, drive state, turn signals
- Debug screen: Raw ADC values and percentages
- Screen toggle: Lock + LeftTurn + PTT buttons
- Timeout indicators for VC/BMS communication

---

## Current Rust Implementation Status

### Completed Components (✅)
- **Embassy async runtime** with STM32F429VI support
- **USB logger** for debugging via USB serial
- **SSD1322 display driver** with embedded-graphics integration
- **Basic button handling** with debouncing and toggle support
- **Task architecture** with concurrent display, button, and blinky tasks
- **Clock configuration** at 168 MHz with 25 MHz external oscillator

### Project Structure
```
src/
├── main.rs                    # Entry point, hardware init
├── drivers/
│   ├── buttons/              # Button handling (10 buttons configured)
│   ├── display/              # SSD1322 driver
│   └── usb/                  # USB serial logger
└── tasks/
    ├── display.rs            # Display animation task
    ├── buttons.rs            # Button monitoring task
    └── blinky.rs            # LED heartbeat task
```

### Current Limitations (❌)
- No networking/Ethernet support
- No ADC implementation for pedals
- No protobuf message handling
- No LED output control (except single blinky)
- Display only shows demo animation
- No vehicle state management
- No persistent configuration

---

## Migration Requirements

### Critical Components (Must Have)
1. **Ethernet Networking**
   - UDP sockets on port 4001 (receive), 3001/2001 (send)
   - Static IP configuration (192.168.0.30)
   - RMII PHY interface configuration

2. **Protobuf Messaging**
   - DataMessage, SW_State, VC_State, BMS_State structures
   - Serialization/deserialization for network communication

3. **ADC for Pedals**
   - Two channels for throttle/brake
   - Low-pass filtering (IIR, α=0.5)
   - Calibration with dead zones

### Important Components
4. **Enhanced Display**
   - Main telemetry screen with vehicle data
   - Debug screen with pedal values
   - Font system (12x16, 48x64)
   - Screen toggle mechanism

5. **Complete LED System**
   - 10 button LEDs with state tracking
   - 4 system status LEDs

### Enhanced Features
6. **Animations & Indicators**
   - Turn signal blinking (500ms)
   - Timeout indicators for network health
   - BMS strobe pattern

7. **State Management**
   - Thread-safe global state
   - Mutex-protected shared data

### Safety Features
8. **System Reliability**
   - Watchdog timer
   - Fault handlers
   - Persistent configuration storage

---

## Proposed Rust Architecture

### Design Principles
1. **Async-first** using Embassy's async/await
2. **Zero allocation** where possible (heapless collections)
3. **Type safety** leveraging Rust's type system
4. **Modular** with clear separation of concerns
5. **Testable** with dependency injection and traits

### Project Structure
```
src/
├── main.rs                    # Entry point, hardware init, task spawning
├── lib.rs                     # Public API and module exports
├── config.rs                  # System-wide constants and configuration
│
├── drivers/                   # Low-level hardware interfaces
│   ├── buttons/              # Button input handling
│   │   ├── mod.rs           # ButtonId, ButtonEvent, ButtonState
│   │   └── debounce.rs      # Debouncing logic
│   ├── pedals/              # ADC for throttle/brake
│   │   ├── mod.rs           # PedalInputs, PedalState
│   │   ├── adc.rs           # ADC driver implementation
│   │   └── calibration.rs   # Calibration and dead zone logic
│   ├── leds/                # LED outputs
│   │   ├── mod.rs           # LED control interface
│   │   ├── button_leds.rs   # 10 button LED control
│   │   └── status_leds.rs   # 4 system status LEDs
│   ├── display/             # SSD1322 OLED
│   │   ├── screens/         # Different display modes
│   │   │   ├── telemetry.rs # Main telemetry screen
│   │   │   └── debug.rs     # Debug/pedal values screen
│   │   └── fonts/           # Font data and rendering
│   ├── network/             # Ethernet communication
│   │   ├── ethernet.rs      # PHY initialization
│   │   ├── udp.rs          # UDP socket management
│   │   └── config.rs       # IP addresses, ports
│   └── usb/                # USB logger (existing)
│
├── protocol/                # Communication protocols
│   ├── messages.proto      # Protobuf definitions
│   ├── messages.rs         # Generated protobuf code
│   └── codec.rs           # Serialization helpers
│
├── state/                  # Application state management
│   ├── vehicle.rs         # VehicleState struct
│   ├── steering.rs        # SteeringState struct
│   └── manager.rs         # Thread-safe state manager
│
├── tasks/                  # Async tasks
│   ├── steering_wheel.rs  # Main 50ms update task
│   ├── network_recv.rs    # UDP receive task
│   ├── network_send.rs    # Telemetry broadcast task
│   ├── display.rs         # Display update task
│   ├── leds.rs           # LED control task
│   └── watchdog.rs       # System health monitor
│
└── utils/                  # Utility functions
    ├── filters.rs         # IIR filter implementation
    ├── timeout.rs         # Timeout tracking
    └── math.rs           # Interpolation, mapping
```

### Core Architecture Patterns

#### Shared State Management
```rust
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub struct SharedState {
    inner: Mutex<CriticalSectionRawMutex, StateData>,
}

pub struct StateData {
    pub steering: SteeringState,
    pub vehicle: VehicleState,
    pub bms: BmsState,
    pub last_vc_message: Instant,
    pub last_bms_message: Instant,
}
```

#### Event Channels
```rust
use embassy_sync::channel::{Channel, Sender, Receiver};

pub static BUTTON_EVENTS: Channel<CriticalSectionRawMutex, ButtonEvent, 16> = Channel::new();
pub static NETWORK_MESSAGES: Channel<CriticalSectionRawMutex, NetworkMessage, 8> = Channel::new();
```

#### Hardware Abstraction
```rust
pub trait PedalInput {
    async fn read_throttle(&mut self) -> u16;
    async fn read_brake(&mut self) -> u16;
    fn calibrate(&mut self, throttle_max: u16, brake_max: u16);
}
```

### Task Architecture
```rust
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let state = SharedState::new();

    // Spawn concurrent tasks
    spawner.spawn(steering_wheel_task(state.clone(), buttons, pedals)).unwrap();
    spawner.spawn(network_recv_task(state.clone(), network.clone())).unwrap();
    spawner.spawn(network_send_task(state.clone(), network.clone())).unwrap();
    spawner.spawn(display_task(state.clone(), display)).unwrap();
    spawner.spawn(led_task(state.clone())).unwrap();
    spawner.spawn(watchdog_task()).unwrap();
}
```

### Technology Choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Async Runtime | Embassy | STM32 integration, low overhead |
| Network Stack | embassy-net + smoltcp | Async support, lightweight |
| Protobuf | prost | Most popular, good codegen |
| State Management | Shared Mutex | Simple, adequate for our needs |
| Display Graphics | embedded-graphics | De facto standard |

---

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)
- [x] System architecture design
- [ ] Create project directory structure
- [ ] Set up core state structures
- [ ] Implement ADC driver for pedals
- [ ] Add low-pass filtering
- [ ] Implement calibration logic

### Phase 2: Hardware Enhancement (Week 1-2)
- [ ] Enhance button system with LED association
- [ ] Add LED output control for all 10 button LEDs
- [ ] Implement 4 system status LEDs
- [ ] Complete debouncing implementation

### Phase 3: Communication (Week 2)
- [ ] Set up protobuf with prost
- [ ] Port message definitions to .proto file
- [ ] Configure embassy-net for Ethernet
- [ ] Implement UDP socket communication
- [ ] Create network receive task
- [ ] Add telemetry broadcast task

### Phase 4: Display & UI (Week 3)
- [ ] Port font rendering (12x16, 48x64)
- [ ] Create main telemetry screen
- [ ] Implement debug screen
- [ ] Add screen toggle logic
- [ ] Implement timeout indicators
- [ ] Add turn signal animations

### Phase 5: Integration (Week 4)
- [ ] Create main steering wheel update task
- [ ] Connect all components
- [ ] Add watchdog timer
- [ ] Implement fault handlers
- [ ] Create integration tests
- [ ] Performance optimization

### Quick Wins to Start
1. **ADC Setup** - Get pedal inputs working first
2. **LED Outputs** - Complete button LED control
3. **Basic Protobuf** - Start with simple messages
4. **Display Screens** - Port main telemetry display

---

## Technical Details

### Pin Assignments

#### Buttons (Active Low with Pull-up)
| Button | Pin | Mode | LED Pin |
|--------|-----|------|---------|
| Cruise Down | PD12 | Momentary | PD13 |
| Cruise Up | PE14 | Momentary | PE15 |
| Reverse | PE0 | Momentary | PE1 |
| Push-to-Talk | PE4 | Momentary | PE5 |
| Horn | PD14 | Momentary | PD15 |
| Power Save | PE2 | Momentary | PE3 |
| Rearview | PE8 | Momentary | PE9 |
| Left Turn | PE12 | Toggle | PE13 |
| Right Turn | PE6 | Toggle | PE7 |
| Lock | PE10 | Toggle | PE11 |

#### System LEDs
| LED | Pin | Color |
|-----|-----|-------|
| Status 1 | PD8 | White |
| Status 2 | PD9 | Blue |
| Status 3 | PD10 | Green |
| Status 4 | PD11 | Red |

#### Display (SSD1322)
| Signal | Pin | Function |
|--------|-----|----------|
| SCLK | PB3 | SPI Clock |
| MOSI | PB5 | SPI Data |
| CS | PA15 | Chip Select |
| DC | PB6 | Data/Command |
| Reset | PD7 | Hardware Reset |

#### ADC
| Channel | Pin | Function |
|---------|-----|----------|
| ADC1_IN9 | PB1 | Throttle |
| ADC1_IN8 | PB0 | Brake |

#### Ethernet (RMII)
| Signal | Pin |
|--------|-----|
| CRS_DV | PA7 |
| RefClk | PA1 |
| RXD0 | PC4 |
| RXD1 | PC5 |
| TxEn | PB11 |
| TxD0 | PB12 |
| TxD1 | PB13 |
| MDC | PC1 |
| MDIO | PA2 |
| PHY_RST | PD15 |

### Network Configuration
```rust
pub const IP_ADDRESS: IpAddress = IpAddress::v4(192, 168, 0, 30);
pub const NETMASK: IpAddress = IpAddress::v4(255, 255, 255, 0);
pub const GATEWAY: IpAddress = IpAddress::v4(192, 168, 0, 1);

pub const VC_ADDRESS: SocketAddr = SocketAddr::new(
    IpAddress::v4(192, 168, 0, 20), 3001
);
pub const BMS_ADDRESS: SocketAddr = SocketAddr::new(
    IpAddress::v4(192, 168, 0, 10), 2001
);
pub const RECEIVE_PORT: u16 = 4001;
pub const TELEMETRY_PORT: u16 = 6000;
```

### Timing Requirements
| Task | Frequency | Priority |
|------|-----------|----------|
| Steering Update | 50ms (20 Hz) | High |
| Display Refresh | 33ms (30 Hz) | Medium |
| Network Receive | Event-driven | High |
| Telemetry Send | 1000ms (1 Hz) | Low |
| LED Update | Continuous | Low |

### Performance Targets
- Button response latency: < 100ms
- Network message latency: < 10ms
- Display update rate: 30 FPS minimum
- ADC sampling rate: 20 Hz minimum
- System idle time: > 50%

---

## Testing Strategy

### Unit Tests
- Filter algorithms
- Calibration logic
- State management
- Protocol serialization

### Integration Tests
- Button → LED response
- ADC → Network transmission
- Network → Display update
- Full message round-trip

### Hardware-in-Loop Tests
- Pedal calibration sequence
- Button debouncing validation
- Display refresh rate
- Network throughput

### System Tests
- Stress testing with all tasks active
- Network disconnection handling
- Power cycling resilience
- Memory usage monitoring

---

## Risk Mitigation

### Technical Risks
1. **Ethernet driver complexity**
   - Mitigation: Use proven embassy-net implementation
   - Fallback: Implement simpler UDP-only stack

2. **Real-time constraints**
   - Mitigation: Profile early and often
   - Fallback: Reduce display update rate if needed

3. **Memory constraints**
   - Mitigation: Static allocation, no heap
   - Fallback: Optimize data structures

### Schedule Risks
1. **Protobuf integration complexity**
   - Mitigation: Start with manual serialization
   - Fallback: Use simpler binary format

2. **Display performance**
   - Mitigation: Pre-render static elements
   - Fallback: Simplify UI if needed

---

## Success Criteria

### Functional Requirements
- [ ] All 10 buttons respond correctly with debouncing
- [ ] Pedals provide smooth 0-1 values
- [ ] Display shows vehicle telemetry clearly
- [ ] Network messages sent/received reliably
- [ ] All LEDs controllable

### Performance Requirements
- [ ] Button latency < 100ms
- [ ] Display refresh > 30 FPS
- [ ] Network latency < 10ms
- [ ] CPU usage < 50%
- [ ] Zero memory leaks

### Reliability Requirements
- [ ] Runs continuously for 24+ hours
- [ ] Recovers from network disconnection
- [ ] Handles all error conditions gracefully
- [ ] Watchdog prevents system hang

---

## Appendix: Message Definitions

### Protobuf Schema (Simplified)
```proto
syntax = "proto3";

message DataMessage {
    SW_State sw_state = 1;
    VC_State vc_state = 2;
    BMS_State bms_state = 3;
}

message SW_State {
    SteerButtonState button_state = 1;
    float throttle = 2;
    float brake = 3;
    uint32 screen = 4;
    uint32 time_tracker_VC = 5;
    uint32 time_tracker_BMS = 6;
}

message SteerButtonState {
    bool cruise_down_on = 1;
    bool cruise_up_on = 2;
    bool reverse_on = 3;
    bool horn_on = 4;
    bool lock_on = 5;
    bool rearview_on = 6;
    bool power_save_on = 7;
    bool ptt_on = 8;
    bool left_turn_on = 9;
    bool right_turn_on = 10;
    uint32 led_state = 11;
}

message VC_State {
    float speed = 1;
    uint32 drive_mode = 2;
    Lights vc_lights = 3;
    // ... additional fields
}

message BMS_State {
    float voltage = 1;
    float current = 2;
    uint32 flags = 3;
    // ... additional fields
}
```

---

## References

- [Embassy Documentation](https://embassy.dev)
- [STM32F429 Reference Manual](https://www.st.com/resource/en/reference_manual/rm0090)
- [SSD1322 Datasheet](https://www.solomon-systech.com/en/product/advanced-display/oled-display-driver-ic/ssd1322/)
- [Original C Codebase](../sunburnt-2023/onboard/embedded/steering_wheel)

---

*Document Version: 1.0*
*Last Updated: November 2, 2025*
*Author: Claude Assistant with Jeremy*