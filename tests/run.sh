#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
KERNEL_ISO="$PROJECT_DIR/dist/x86_64/kernel.iso"
TIMEOUT=5

echo "=== Building kernel (in Docker) ==="
cd "$PROJECT_DIR"
docker run --rm -i -v "$PROJECT_DIR/:/root/env" rust-kernel-env bash -c "cd /root/env && make build"

echo "=== Running QEMU boot test ==="

# Check kernel ISO exists
if [ ! -f "$KERNEL_ISO" ]; then
    echo "ERROR: Kernel ISO not found at $KERNEL_ISO"
    exit 1
fi

# Run QEMU and check it starts without immediate crash
QMP_SOCKET="$SCRIPT_DIR/qemu_qmp.sock"

rm -f "$QMP_SOCKET"

timeout $TIMEOUT qemu-system-x86_64 \
    -cdrom "$KERNEL_ISO" \
    -display none \
    -monitor none \
    -qmp "unix:$QMP_SOCKET,server,nowait" \
    >/dev/null 2>&1 &

QEMU_PID=$!
sleep 2

if kill -0 $QEMU_PID 2>/dev/null; then
    echo "=== TEST PASSED: Kernel booted successfully ==="
    RESULT=0
else
    echo "=== TEST FAILED: QEMU crashed or failed to start ==="
    RESULT=1
fi

kill $QEMU_PID 2>/dev/null || true
rm -f "$QMP_SOCKET"

exit $RESULT
