# Alias for build-x86_64
.PHONY: build
build: build-x86_64

# Directories
ASM_SRC_DIR := src/implementation/x86_64/boot
ASM_SRCS    := $(wildcard $(ASM_SRC_DIR)/*.asm)
ASM_OBJS    := $(patsubst $(ASM_SRC_DIR)/%.asm, build/x86_64/%.o, $(ASM_SRCS))

RUST_SRC_ALL := $(shell find src/implementation/kernel -name '*.rs')
RUST_LIB     := target/x86_64-unknown-none/release/librust_kernel.a

LINKER_SCRIPT := targets/x86_64/linker.ld
KERNEL_BIN    := dist/x86_64/kernel.bin
ISO_DIR       := targets/x86_64/iso
GRUB_CFG      := $(ISO_DIR)/boot/grub/grub.cfg

# Compile assembly to object files
build/x86_64/%.o: src/implementation/x86_64/boot/%.asm
	mkdir -p $(dir $@)
	nasm -f elf64 $< -o $@

# Build Rust staticlib
$(RUST_LIB): $(RUST_SRC_ALL)
	cargo build --release --target x86_64-unknown-none

# Link all objects + Rust lib into kernel.bin
$(KERNEL_BIN): $(ASM_OBJS) $(RUST_LIB) $(LINKER_SCRIPT)
	mkdir -p $(dir $@)
	x86_64-elf-ld -n -T $(LINKER_SCRIPT) -o $@ $(ASM_OBJS) $(RUST_LIB)

# Copy kernel.bin to ISO dir
copy-kernel: $(KERNEL_BIN)
	cp $(KERNEL_BIN) $(ISO_DIR)/boot/kernel.bin

# Build ISO
iso: copy-kernel
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso $(ISO_DIR)

.PHONY: build-x86_64 clean iso copy-kernel
build-x86_64: $(KERNEL_BIN) iso

clean:
	rm -rf build dist target




docker-shell:
	docker run --rm -it -v ~/Projects/rust-kernel/:/root/env rust-kernel-env bash

build-docker:
	cd ~/Projects/rust-kernel/buildenv && docker build -t rust-kernel-env .


run-kernel:
	qemu-system-x86_64 -cdrom dist/x86_64/kernel.iso

