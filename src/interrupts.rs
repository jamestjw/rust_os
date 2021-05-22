use crate::gdt;
use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_idt() {
    // Loads the IDT using the `lidt` instruction
    IDT.load();
}

// Breakpoint handler is invoked when the `int3` instruction is executed
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// Double fault handler is invoked when an exception occurs and
// fails to invoke the corresponding handler.
//
// This is a diverging function as the x86_64 architecture does not
// permit returning from a double fault exception.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    // The error code is always 0 for double faults
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// If execution continues, we verify that the breakpoint
// handler is working correctly.
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
