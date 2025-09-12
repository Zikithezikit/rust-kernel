
# Directories
ASM_SRC_DIR := src/implementation/x86_64/boot
ASM_SRCS := $(wildcard $(ASM_SRC_DIR)/*.asm)
ASM_OBJS := $(patsubst $(ASM_SRC_DIR)/%.asm, build/x86_64/%.o, $(ASM_SRCS))


# Only compile main.rs, which includes other modules
RUST_MAIN := src/implementation/kernel/main.rs
RUST_OBJ := build/kernel/main.o

LINKER_SCRIPT := targets/x86_64/linker.ld
KERNEL_BIN := dist/x86_64/kernel.bin
ISO_DIR := targets/x86_64/iso
GRUB_CFG := $(ISO_DIR)/boot/grub/grub.cfg

# Compile assembly to object files
build/x86_64/%.o: src/implementation/x86_64/boot/%.asm
	mkdir -p $(dir $@)
	nasm -f elf64 $< -o $@

# Compile only main.rs to object file
$(RUST_OBJ): $(RUST_MAIN)
	mkdir -p $(dir $@)
	rustc --target x86_64-unknown-none -C opt-level=2 --emit=obj -o $@ $<

# Link all objects into kernel.bin
$(KERNEL_BIN): $(ASM_OBJS) $(RUST_OBJ) $(LINKER_SCRIPT)
	mkdir -p $(dir $@)
	x86_64-elf-ld -n -T $(LINKER_SCRIPT) -o $@ $(ASM_OBJS) $(RUST_OBJ)

# Copy kernel.bin to ISO dir
copy-kernel: $(KERNEL_BIN)
	cp $(KERNEL_BIN) $(ISO_DIR)/boot/kernel.bin

# Build ISO
iso: copy-kernel
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso $(ISO_DIR)

.PHONY: build-x86_64 clean iso copy-kernel
build-x86_64: $(KERNEL_BIN) iso

clean:
	rm -rf build dist