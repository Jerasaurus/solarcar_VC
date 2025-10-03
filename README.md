# Solar Car Vehicle Computer

An embedded firmware project for Stanford Solar Car's vehicle computer using the Embassy async framework on STM32F429VI microcontroller.

## Features

- **Async/await runtime** using Embassy executor
- **USB logging** for debugging via USB CDC
- **LED blinky** demonstration on PD8
- **External 25MHz oscillator** configuration
- **Optimized clock configuration** for 168MHz system clock and 48MHz USB

## Hardware Requirements

- STM32F429VI microcontroller
- 25MHz external oscillator
- USB connection on PA11/PA12
- LED connected to PD8

## Building

Build the project using cargo:

```bash
cargo build --release 
```

## Flashing

Flash the firmware to your target using cargo run. Make sure device is in USB DFU

```bash
cargo run --release
```

## USB Logging

Connect to the USB   port to view debug logs. The device will enumerate as a USB serial device when connected.

```bash
cu -l /dev/tty.usbmodem11101 -s 115200
#replace usbmodem11101 with your device found in ls /dev/tty.usbmodem*
```

## Dependencies

- Embassy framework for async embedded development
- defmt for efficient logging
- cortex-m for ARM Cortex-M specific functionality

## License

MIT OR Apache-2.0