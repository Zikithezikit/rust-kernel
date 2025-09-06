# How to compile the kernel

> Firstly build the docker image. This might take a while at the first time.
```bash
cd ~/my-kernel/buildenv && docker build -t my-kernel-env .
```

> Now run the container.
```bash 
cd ~/my-kernel/buildenv && docker run --rm -it -v ~/my-kernel:/root/env my-kernel-env bash
```


> In the ocntainer run the build command.
```bash
make build-x86_64
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

