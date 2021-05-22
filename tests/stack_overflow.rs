// In this test, we invoke a stack overflow without defining a
// page fault exception handler. The stack overflow itself triggers
// a page fault, and a failure to find the corresponding exception
// handler should trigger a double fault exception.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use rust_os::serial_print;
use rust_os::{exit_qemu, serial_println, QemuExitCode};
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

// Building a custom IDT for this test so that we can have
// a custom double fault exception handler. We also do not
// define a page fault handler, which leads to a double fault
// when a stack overflow occurs.
lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rust_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    rust_os::gdt::init();
    // Setup custom IDT with double fault handler that exits
    // QEMU with a successful exit code.
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow!");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // On each call, the return address is pushed
    volatile::Volatile::new(0).read(); // Prevent tail recursion optimization
}

fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
