#![no_std] // don't link standard library
#![no_main] // disable Rust-level entry points, hence we won't need a main function
#![feature(custom_test_frameworks)] // collects functions annotated with #[test_case] attribute
#![test_runner(rust_os::test_runner)]
// custom_test_frameworks generates a main function that calls
// test_runner, but this function is ignored since we use the
// #[no_main] attribute. so we reexport the function and
// call it ourselves
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::println;
use rust_os::task::keyboard;
use rust_os::task::{executor::Executor, Task};

// Panic handler should never return, we will
// let it loop infinitely for now
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    rust_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

// Overriding the entrypoint of the "crt0" C
// runtime library which the linker uses
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::allocator;
    use rust_os::memory;
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");

    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async_number: {}", number);
}
