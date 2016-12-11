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