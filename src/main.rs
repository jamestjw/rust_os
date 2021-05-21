#![no_std] // don't link standard library
#![no_main] // disable Rust-level entry points, hence we won't need a main function
#![feature(custom_test_frameworks)] // collects functions annotated with #[test_case] attribute
#![test_runner(rust_os::test_runner)]
// custom_test_frameworks generates a main function that calls
// test_runner, but this function is ignored since we use the
// #[no_main] attribute. so we reexport the function and
// call it ourselves
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

// Panic handler should never return, we will
// let it loop infinitely for now
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

// Overriding the entrypoint of the "crt0" C
// runtime library
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    rust_os::init();

    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("no crash!");

    loop {}
}
