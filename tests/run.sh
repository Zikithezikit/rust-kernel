#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
KERNEL_ISO="$PROJECT_DIR/dist/x86_64/kernel.iso"

QEMU_TIMEOUT=10
BOOT_WAIT_SECONDS=2

VERBOSE=${VERBOSE:-0}

if [ "$VERBOSE" = "1" ]; then
    echo "=== Building kernel ==="
    cd "$PROJECT_DIR"
    make build
else
    cd "$PROJECT_DIR"
    echo -n "Compiling"
    make build >/dev/null 2>&1 &
    BUILD_PID=$!
    while kill -0 $BUILD_PID 2>/dev/null; do
        echo -n "."
        sleep 0.5
    done
    wait $BUILD_PID
    echo ""
fi

echo "=== Running QEMU boot test ==="

if [ ! -f "$KERNEL_ISO" ]; then
    echo "ERROR: Kernel ISO not found at $KERNEL_ISO"
    exit 1
fi

LOG_FILE="$SCRIPT_DIR/qemu_output.log"
rm -f "$LOG_FILE"

timeout $QEMU_TIMEOUT qemu-system-x86_64 \
    -cdrom "$KERNEL_ISO" \
    -serial file:"$LOG_FILE" \
    -display none \
    -monitor none \
    2>/dev/null &

QEMU_PID=$!
sleep $BOOT_WAIT_SECONDS

if kill -0 $QEMU_PID 2>/dev/null; then
    kill $QEMU_PID 2>/dev/null || true
fi

wait $QEMU_PID 2>/dev/null || true

OUTPUT=""
if [ -f "$LOG_FILE" ]; then
    OUTPUT=$(cat "$LOG_FILE")
    rm -f "$LOG_FILE"
fi

echo "=== Serial output ==="
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "Serial initialized"; then
    echo "=== TEST PASSED ==="
    exit 0
else
    echo "=== TEST FAILED ==="
    exit 1
fi
