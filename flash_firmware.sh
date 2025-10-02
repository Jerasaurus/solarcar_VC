#!/bin/bash
set -e
BINARY=$1
cargo objcopy --release --bin $(basename "$BINARY") -- -O binary "${BINARY}.bin"
dfu-util -a 0 -s 0x08000000:leave -D "${BINARY}.bin"