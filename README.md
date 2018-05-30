# cervus

A WebAssembly subsystem for Linux.

![Screenshot](https://i.imgur.com/QFvUibQ.png)

## What is it?

Cervus implements a WebAssembly "usermode" on top of the Linux kernel (which tries to follow the [CommonWA](https://github.com/CommonWA/cwa-spec) specification), enabling wasm applications to run directly in ring 0, while still ensuring safety and security.

## But why?

- Managed execution (making it possible to perform optimizations based on tracing/partial evaluation)
- Avoid performance overhead introduced by system calls & address space switches
- Zero-copy I/O is possible

## Things that are working and not working

**Working:**

- An interpreter based on [HexagonE](https://github.com/losfair/hexagon-e)
- Binary translation & loading based on [wasm-core](https://github.com/losfair/wasm-core)
- Most of CommonWA ("everything is a URL", file I/O, command-line arguments)
- IPC (only broadcast supported by now, with URL prefix `ipc-broadcast://`)

**Not working:**

- Floating point
- JIT
- Everything else

## Build

### Kernel module

Requirements:

- xargo
- latest nightly rust
- kernel headers
- gnu make & gcc

```
./build_all.sh
sudo insmod glue/cervus.ko
```

### Loader (cvctl)

This installs the `cvload` and `cvrun` binaries:

```
cd cvctl
cargo install
```

### Applications

Cervus implements most of [CommonWA](https://github.com/CommonWA/cwa-spec) (tracked at [#2](https://github.com/cervus-v/cervus/issues/2)), whose examples can be found at [cwa-rs/examples](https://github.com/CommonWA/cwa-rs/tree/master/examples).

For example, to build and run the `cat` example:

```
sudo chmod 666 /dev/cvctl
cd cwa-rs
cargo build --target wasm32-unknown-unknown --release --example cat
cvrun target/wasm32-unknown-unknown/release/examples/cat.wasm file:///etc/lsb-release
```

To launch an IPC broadcast sender and then read from it:

```
cargo build --target wasm32-unknown-unknown --release --example broadcast_sender
cvrun target/wasm32-unknown-unknown/release/examples/broadcast_sender.wasm your_broadcast
```

(in another terminal)

```
cvrun target/wasm32-unknown-unknown/release/examples/cat.wasm ipc-broadcast://your_broadcast | dd of=/dev/null bs=4K
```

## Contribute

I'm busy with my College Entrance Examination until ~June 10, 2018, before which I cannot actively maintain this project. However, there are a few things that can be relatively easily worked on:

- A JIT based on Cretonne

Since Cretonne supports `no_std`, this should be relatively easy compared to other JIT approaches.

Interface with the rest of the system by implementing the `Backend` trait, for which the interpreter-based backend located in `src/backend/hexagon_e` is a good example to start with.

- Network API

Blocking network APIs can be added as virtual system calls.

## License

Cervus itself has to use the GPL 2.0 license because it links to the Linux kernel. However, user code that runs on Cervus is not limited by this.
