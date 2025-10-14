#!/bin/bash
set -e

BINARY=$1
MAX_RETRIES=20
RETRY_DELAY=5

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Building binary..."
cargo objcopy --release --bin $(basename "$BINARY") -- -O binary "${BINARY}.bin"

echo "Attempting to flash firmware..."

# Try to flash with retries
for i in $(seq 1 $MAX_RETRIES); do
    echo -e "${YELLOW}Attempt $i of $MAX_RETRIES${NC}"

    # Try to run dfu-util
    if dfu-util -a 0 -s 0x08000000:leave -D "${BINARY}.bin" 2>&1; then
        echo -e "${GREEN}✓ Firmware flashed successfully!${NC}"
        exit 0
    else
        # Check if device is not in DFU mode
        if ! dfu-util -l 2>&1 | grep -q "Found DFU"; then
            echo -e "${RED}⚠ Device not found in DFU mode!${NC}"
            echo -e "${YELLOW}Please put your device into DFU mode:${NC}"
            echo "  1. Hold down the BOOT button"
            echo "  2. Press and release the RESET button"
            echo "  3. Release the BOOT button"
            echo ""

            if [ $i -lt $MAX_RETRIES ]; then
                echo -e "Retrying in $RETRY_DELAY seconds... (Press Ctrl+C to cancel)"
                sleep $RETRY_DELAY
            else
                echo -e "${RED}✗ Failed to flash after $MAX_RETRIES attempts${NC}"
                echo "Please ensure your device is properly connected and in DFU mode."
                exit 1
            fi
        else
            # Some other error occurred
            echo -e "${RED}✗ Flash failed but device is in DFU mode${NC}"
            echo "There may be another issue with the flashing process."

            if [ $i -lt $MAX_RETRIES ]; then
                echo -e "Retrying in $RETRY_DELAY seconds... (Press Ctrl+C to cancel)"
                sleep $RETRY_DELAY
            else
                echo -e "${RED}✗ Failed to flash after $MAX_RETRIES attempts${NC}"
                exit 1
            fi
        fi
    fi
done