#![no_std] // don't link standard library
#![no_main] // disable Rust-level entry points, hence we won't need a main function

use core::panic::PanicInfo;

// Panic handler should never return, we will
// let it loop infinitely for now
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Overriding the entrypoint of the "crt0" C
// runtime library
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
