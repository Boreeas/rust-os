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
    set_color!(RED);
    print!("\nERROR: ");
    set_color!(WHITE);
    println!("division by zero");
    set_color!(LIGHT_GRAY);
    println!("{:#?}", stack_frame);
    
    loop {}
}

pub extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    set_color!(RED);
    print!("\nERROR: ");
    set_color!(WHITE);
    println!("invalid opcode at {:#x}", stack_frame.instruction_pointer);
    set_color!(LIGHT_GRAY);
    println!("{:#?}", stack_frame);

    loop {}
}

pub extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, errno: u64) {
    use ::x86::shared::control_regs;

    set_color!(RED);
    print!("\nERROR: ");
    set_color!(WHITE);
    println!("page fault trying to access 0x{:x} ({:?})", 
        unsafe { control_regs::cr2() },
        PageFaultErrorCode::from_bits(errno).unwrap());
    set_color!(LIGHT_GRAY);
    println!("{:#?}", stack_frame);

    loop {}
}

pub extern "C" fn breakpoint_handler(stack_frame: &ExceptionStackFrame) {
    set_color!(RED);
    print!("\nBREAKPOINT: ");
    set_color!(WHITE);
    println!("At instruction {:#x}", stack_frame.instruction_pointer);
    set_color!(LIGHT_GRAY);
    println!("{:#?}", stack_frame);

    println!("Press any key to continue");
    ::keyboard::next_key();
}
