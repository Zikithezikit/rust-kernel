# Build targets
.PHONY: build build-kernel clean iso copy-kernel docker-shell build-docker run-kernel run-tests run-tests-verbose

# Direct build (requires cross-compiler tools - use inside Docker)
build-kernel: $(KERNEL_BIN) iso

# Host build - uses Docker to run build-kernel
build: clean
ifneq ($(IS_DOCKER),true)
	@docker run --rm -i -v $$(pwd)/:/root/env rust-kernel-env bash -c "cd /root/env && IS_DOCKER=true make build-kernel"
else
	@$(MAKE) build-kernel
endif

# Directories
ASM_SRC_DIR := src/x86_64/boot
ASM_SRCS    := $(wildcard $(ASM_SRC_DIR)/*.asm)
ASM_OBJS    := $(patsubst $(ASM_SRC_DIR)/%.asm, build/x86_64/%.o, $(ASM_SRCS))

RUST_SRC_ALL := $(shell find src/kernel -name '*.rs')
RUST_LIB     := target/x86_64-unknown-none/release/librust_kernel.a

LINKER_SCRIPT := targets/x86_64/linker.ld
KERNEL_BIN    := dist/x86_64/kernel.bin
ISO_DIR       := targets/x86_64/iso

# Compile assembly to object files
build/x86_64/%.o: src/x86_64/boot/%.asm
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

clean:
	docker run --rm -i -v $$(pwd)/:/root/env rust-kernel-env bash -c "rm -rf build dist target"

docker-shell:
	docker run --rm -it -v $$(pwd)/:/root/env rust-kernel-env bash

build-docker:
	cd buildenv && docker build -t rust-kernel-env .

run-kernel:
	qemu-system-x86_64 -cdrom dist/x86_64/kernel.iso

run-tests:
	bash tests/run.sh

run-tests-verbose:
	VERBOSE=1 bash tests/run.sh
