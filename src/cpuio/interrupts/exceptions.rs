use core::fmt;

#[repr(C,packed)]
#[derive(Copy,Clone,Debug)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64
}

#[repr(C,packed)]
#[derive(Copy,Clone,Debug)]
pub struct ErrorCode(u64);

impl ErrorCode {
    fn external(&self) -> bool {
        self.0 & 1 == 1
    }

    fn location(&self) -> ::DescriptorTable {
        match (self.0 >> 1) & 0b11 {
            0 => ::DescriptorTable::GDT,
            1 => ::DescriptorTable::IDT,
            2 => ::DescriptorTable::LDT,
            _ => panic!("Never happens")
        }
    }

    fn selector_index(&self) -> u64 {
        self.0 >> 3
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "n/a")
        }

        if self.external() {
            write!(f, "External ")?;
        }
        write!(f, "Error in {:?} (at index {:#x})", self.location(), self.selector_index())
    }
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

macro_rules! fail {
    () => {
        println!("\n\\{},{};Can't recover\\{},{};", 
            BLACK as u8, RED as u8, RED as u8, BLACK as u8);
        unsafe { asm!("hlt") }
    }
}
pub extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nERROR: {}division by zero", RED, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);
    
    fail!();
}

pub extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nERROR: {}invalid opcode at {:#x}",
        RED, WHITE, stack_frame.instruction_pointer);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, errno: u64) {
    use ::x86::shared::control_regs;

    println!("{}\nERROR: {}page fault trying to access 0x{:x} ({:?})", 
        RED, WHITE,
        unsafe { control_regs::cr2() },
        PageFaultErrorCode::from_bits(errno).unwrap());
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn breakpoint_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nBREAKPOINT: {}At instruction {:#x}", 
        RED, WHITE, stack_frame.instruction_pointer);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    println!("Press any key to continue");
    ::keyboard::next_key();
}

pub extern "C" fn debug_exception_handler(stack_frame: &ExceptionStackFrame) {
    let dr6: u64;
    unsafe { asm!("mov $0, dr6" : "=r"(dr6) ::: "intel", "volatile"); }

    println!("{}\nDEBUG: {}Triggered (DR6: {})", RED, WHITE, dr6);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);
    
    fail!();
}

pub extern "C" fn overflow_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nERROR: {}Overflow", RED, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn out_of_bounds_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nERROR: {}Out of bounds", RED, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn device_not_available_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nERROR: {}Couldn't execute FP instruction at {}{}{} (Device not available)", RED, WHITE, LIGHT_GRAY, stack_frame.instruction_pointer, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame, _: u64) {
    println!("{}\nFATAL: {}Double fault", RED, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn missing_segment_handler(stack_frame: &ExceptionStackFrame, error: ErrorCode) {
    println!("{}\nERROR: {}Missing segment (Details: {}{}{})", RED, WHITE, LIGHT_GRAY, error, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn stack_fault_handler(stack_frame: &ExceptionStackFrame, error: ErrorCode) {
    println!("{}\nERROR: {}Stack fault (Details: {}{}{})", RED, WHITE, LIGHT_GRAY, error, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn general_protection_fault_handler(stack_frame: &ExceptionStackFrame, error: ErrorCode) {
    println!("{}\nERROR: {}General Protection Exception (Error: {}{}{})", RED, WHITE, LIGHT_GRAY, error, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn floating_point_error_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nERROR: {}Error executing floating-point instruction", RED, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn alignment_check_handler(stack_frame: &ExceptionStackFrame, error: ErrorCode) {
    println!("{}\nERROR: {}Alignment checking requested, operand at {}{:#x}{} is not aligned", RED, WHITE, LIGHT_GRAY, stack_frame.instruction_pointer, WHITE);
    println!("Externally triggered: {}", error.external());
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn machine_check_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nFATAL: {}Machine Check or Bus Error", RED, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn simd_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nERROR: {}Error executing SIMD instruction", RED, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}

pub extern "C" fn virtualization_error_handler(stack_frame: &ExceptionStackFrame) {
    println!("{}\nERROR: {}Virtualization error", RED, WHITE);
    println!("{}{:#?}", LIGHT_GRAY, stack_frame);

    fail!();
}