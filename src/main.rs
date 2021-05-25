#![no_std] // don't link standard library
#![no_main] // disable Rust-level entry points, hence we won't need a main function
#![feature(custom_test_frameworks)] // collects functions annotated with #[test_case] attribute
#![test_runner(rust_os::test_runner)]
// custom_test_frameworks generates a main function that calls
// test_runner, but this function is ignored since we use the
// #[no_main] attribute. so we reexport the function and
// call it ourselves
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::println;

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
    use rust_os::memory;
    // to access the translate_addr method of this Trait
    use x86_64::structures::paging::Page;
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");

    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // Map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    rust_os::hlt_loop();
}
