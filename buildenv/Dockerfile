FROM randomdude/gcc-cross-x86_64-elf

RUN apt-get update
RUN apt-get upgrade -y
RUN apt-get install -y nasm
RUN apt-get install -y xorriso
RUN apt-get install -y grub-pc-bin

# Install Rust using rustup (for bare metal kernel dev)
RUN apt-get update && apt-get install -y curl
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"
# Add the required target for bare metal x86_64
RUN /root/.cargo/bin/rustup target add x86_64-unknown-none

VOLUME /root/env
WORKDIR /root/env