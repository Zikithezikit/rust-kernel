# How to compile the kernel

> Firstly build the docker image. This might take a while at the first time.
```bash
cd ~/Projects/rust-kernel/buildenv && docker build -t rust-kernel-env .
```

> Run the build command.
```bash
make build
```

And that's it now you should have both the `kernel.iso` and the `kernel.bin` files.

---

# How to run the kernel

> In your shell: (Outside the container)
```bash
qemu-system-x86_64 -cdrom dist/x86_64/kernel.iso 
```

> Or 
```bash
make run-kernel
```

