#![no_std]

#[macro_use]
extern crate rttrust;

use rttrust::{
    ipc::{mutex::Mutex, IpcFlag},
    object::Object,
    thread::Thread,
    Box,
};

#[no_mangle]
pub extern "C" fn rust_main() {
    println!("hello rust");
    kprintf!(b"test kprintf\n\0");
    kprintf!(b"test kprintf %d %s\n\0", 1, b"hi\0");

    let a = Box::new("ALLOC");
    println!("Alloc Test: {:}", *a == "ALLOC");

    println!("Create Mutex");
    let mutex = Mutex::create("mutrust", IpcFlag::Fifo).expect("Failed to create rtt mutex");
    println!("Mutex: {:?}", mutex);

    println!("Create Thread");
    Thread::create_closure(
        "rusttest",
        {
            let mutex = mutex.clone();
            move || {
                println!("Rust Thread test");

                println!("Mutex in closure: {:?}", mutex);

                mutex.take(0).expect("take test failed");

                Thread::delay(1000).unwrap();

                mutex.release().expect("release test failed");

                loop {
                    Thread::delay(5000).ok();
                }
            }
        },
        2048,
        10,
        10,
    )
    .expect("Thread create failed")
    .startup()
    .expect("Thread startup failed");

    Thread::delay(100).unwrap();

    println!("waiting mutex release");

    mutex.take(5000).expect("Mutex take timeout");

    println!("Mutex take successfully");

    println!("Deleting mutex: {}", mutex.get_name().as_ref());

    mutex.delete().unwrap();
}
