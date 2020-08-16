# rttrust

Rust wrapper for [rt-thread](https://github.com/RT-Thread/rt-thread/tree/master)

Usage guide can be found in example folder.

## Supported rt-thread APIs

1. [x] Kernal object
2. [ ] Memory
3. [x] Thread
4. [x] Timer
5. [ ] IPC
   1. [x] Spin lock (`rt_enter_critical`)
   2. [x] Semaphore
   3. [x] Mutex
   4. [ ] Event
   5. [ ] Mailbox
   6. [ ] Message queue
   7. [ ] Signal
6. [ ] Interrupt
7. [ ] Device
   1. [x] Device register
   2. [x] Device access
   3. [ ] UART, PIN, ... (device specific APIs)

## Advanced features

- Allocator
- `print!` and `println!`

## Platforms

- stm32f4xx

## Requirements

- rust toolchain: `nightly`
- rust target: `thumbv7em-none-eabihf`
- [libclang](https://rust-lang.github.io/rust-bindgen/requirements.html)
- `gcc-arm-none-eabi`