#![no_std] // don't link standard library
#![no_main] // disable Rust-level entry points, hence we won't need a main function

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

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
    // The address of the VGA buffer which we use
    // to write to the screen.
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            // Each character is an ASCII byte followed by a color byte

            // Place char byte
            *vga_buffer.offset(i as isize * 2) = byte;
            // Place color byte - a light cyan
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}
