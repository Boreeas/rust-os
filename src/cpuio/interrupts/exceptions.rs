use ::vga_buffer::Color::*;

#[repr(C,packed)]
#[derive(Copy,Clone,Debug)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64
}

bitflags! (
    flags PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0,
        const CAUSED_BY_WRITE = 1 << 1,
        const USER_MODE = 1 << 2,
        const MALFORMED_TABLE = 1 << 3,
        const INSTRUCTION_FETCH = 1 << 4,
    }
);

pub extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) {
    println!("\\{};\nERROR: \\{};division by zero", RED as u8, WHITE as u8);
    println!("\\{};{:#?}", LIGHT_GRAY as u8, stack_frame);
    
    loop {}
}

pub extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    println!("\\{};\nERROR: \\{};invalid opcode at {:#x}",
        RED as u8, WHITE as u8, stack_frame.instruction_pointer);
    println!("\\{};{:#?}", LIGHT_GRAY as u8, stack_frame);

    loop {}
}

pub extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, errno: u64) {
    use ::x86::shared::control_regs;

    println!("\\{};\nERROR: \\{};page fault trying to access 0x{:x} ({:?})", 
        RED as u8, WHITE as u8,
        unsafe { control_regs::cr2() },
        PageFaultErrorCode::from_bits(errno).unwrap());
    println!("\\{};{:#?}", LIGHT_GRAY as u8, stack_frame);

    loop {}
}

pub extern "C" fn breakpoint_handler(stack_frame: &ExceptionStackFrame) {
    println!("\\{};\nBREAKPOINT: \\{};At instruction {:#x}", 
        RED as u8, WHITE as u8, stack_frame.instruction_pointer);
    println!("\\{};{:#?}", LIGHT_GRAY as u8, stack_frame);

    println!("Press any key to continue");
    ::keyboard::next_key();
}
