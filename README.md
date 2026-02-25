# How to compile the kernel

> Firstly build the docker image. This might take a while at the first time.
```bash
cd ~/Projects/rust-kernel/buildenv && docker build -t rust-kernel-env .
```

> Now run the container.
```bash 
cd ~/Projects/rust-kernel/buildenv && docker run --rm -it -v ~/Projects/rust-kernel/:/root/env rust-kernel-env bash
```


> In the container run the build command.
```bash
make build
```

And that's it now you should have both the `kernel.iso` and the `kernel.bin` files.

---
---
---


# How to run the kernel

> In your shell: (Outside the container)
```bash
qemu-system-x86_64 -cdrom dist/x86_64/kernel.iso
```

