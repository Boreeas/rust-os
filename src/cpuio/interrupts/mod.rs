mod exceptions;
use self::exceptions::*;

pub type HandlerFunc = extern "C" fn() -> !;
const NUM_ENTRIES: usize = 256;

#[repr(C,packed)]
#[derive(Clone, Copy)]
// Length: 16 bytes
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    attributes: Attributes,
    offset_middle: u16,
    offset_high: u32,
    __reserved: u32,
}

impl IdtEntry {
    const fn not_set() -> IdtEntry {
        IdtEntry {
            offset_low: 0,
            offset_middle: 0,
            offset_high: 0,
            selector: 0,
            attributes: Attributes::clear(),
            __reserved: 0
        }
    }

    fn new(ptr: HandlerFunc, attributes: Attributes) -> IdtEntry {
        let addr = ptr as usize;
        IdtEntry {
            offset_low: addr as u16,
            offset_middle: (addr >> 16) as u16,
            offset_high: (addr >> 32) as u32,
            selector: 0x08,
            attributes: attributes,
            __reserved: 0 
        }
    }
}

#[repr(C,packed)]
#[derive(Clone,Copy)]
pub struct Attributes(u16);

const DEFAULT: u16        = 0b1100 << 8;
// const CALL_GATE: u8      = 0;
const INTERRUPT_GATE: u16 = 2 << 8;
const TRAP_GATE: u16      = 3 << 8;
const PRESENT: u16        = 1 << 15;

impl Attributes {

    const fn clear() -> Attributes {
        Attributes(0)
    }

    const fn empty() -> Attributes {
        Attributes(DEFAULT)
    }

    const fn new() -> Attributes {
        Attributes(DEFAULT | PRESENT | INTERRUPT_GATE)
    }

    pub fn trap_gate(&mut self) -> &mut Self {
        self.0 = self.0 | TRAP_GATE;
        self
    }

    pub fn interrupt_gate(&mut self) -> &mut Self {
        self.0 = (self.0 & !TRAP_GATE) | INTERRUPT_GATE;
        self
    }

    pub fn call_gate(&mut self) -> &mut Self {
        self.0 = self.0 & !TRAP_GATE;
        self
    }

    /*
    pub fn bits_32(&mut self) -> &mut Self {
        self.0 |= BITS_32;
        self
    }

    pub fn bits_16(&mut self) -> &mut Self {
        self.0 &= !BITS_32;
        self
    }
    */

    pub fn access_level(&mut self, access_level: u16) -> &mut Self {
        if access_level > 3 { 
            panic!("Invalid access level {}", access_level); 
        }

        self.0 = (self.0 & ! 0b11 << 13) | access_level << 13;
        self
    }

    pub fn stack(&mut self, stack: u16) -> &mut Self {
        if stack > 7 {
            panic!("Invalid stack id for interrupt: {}", stack);
        }

        self.0 = (self.0 & !0b111) | stack;
        self
    }
}

macro_rules! save_scratch_registers {
    () => {
        asm!("push rax
              push rcx
              push rdx
              push rsi
              push rdi
              push r8
              push r9
              push r10
              push r11
        " :::: "intel", "volatile");
    }
}

macro_rules! restore_scratch_registers {
    () => {
        asm!("pop r11
              pop r10
              pop r9
              pop r8
              pop rdi
              pop rsi
              pop rdx
              pop rcx
              pop rax" 
              :::: "intel", "volatile");
    }
}

macro_rules! handler {
    ($name:ident) => ({
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_scratch_registers!();

                asm!("mov rdi, rsp
                      add rdi, 9*8 // offset by reg push
                      call $0"
                      :: "i"($name as extern "C" fn(&ExceptionStackFrame))
                      : "rdi" : "intel", "volatile");

                restore_scratch_registers!();
                asm!("iretq" :::: "intel", "volatile");

                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    })
}

macro_rules! handler_with_raw_error {
    ($name:ident) => ({
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_scratch_registers!();

                asm!("mov rsi, [rsp+9*8]
                      mov rdi, rsp
                      add rdi, 10*8
                      sub rsp, 8 // align stack: 9 regs pushed + 6 error qwords
                      call $0
                      add rsp, 8"
                      :: "i"($name as extern "C" fn(&ExceptionStackFrame, u64))
                      : "rdi", "rsi" : "intel");

                restore_scratch_registers!();
                asm!("add rsp, 8 // pop error code
                      iretq"
                      ::: "rsp" : "intel", "volatile");

                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    })
}

macro_rules! handler_with_error {
    ($name:ident) => ({
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_scratch_registers!();

                asm!("mov rsi, [rsp+9*8]
                      mov rdi, rsp
                      add rdi, 10*8
                      sub rsp, 8 // align stack: 9 regs pushed + 6 error qwords
                      call $0
                      add rsp, 8"
                      :: "i"($name as extern "C" fn(&ExceptionStackFrame, ErrorCode))
                      : "rdi", "rsi" : "intel");

                restore_scratch_registers!();
                asm!("add rsp, 8 // pop error code
                      iretq"
                      ::: "rsp" : "intel", "volatile");

                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    })
}

#[repr(C,packed)]
struct IdtRef {
    limit: u16,
    ptr: *const Idt
}

#[repr(C,packed)]
pub struct Idt([IdtEntry; NUM_ENTRIES]);

impl Idt {
    fn new() -> Idt {
        Idt([IdtEntry::not_set(); NUM_ENTRIES])
    }

    pub fn load(&'static self) {
        let idtinfo = IdtRef {
            limit: (20 * NUM_ENTRIES as u16) - 1,
            ptr: self
        };

        unsafe {
            asm!("lidt ($0)"
                : // output
                : "r"(&idtinfo)
                : "memory" // clobbers
                : 
            )
        }
    } 

    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) -> &mut Attributes {
        self.0[entry as usize] = IdtEntry::new(handler, Attributes::new());
        &mut self.0[entry as usize].attributes
    }
}

lazy_static! {
    pub static ref IDT: Idt = {
        let log = log!("  Registering interrupt handlers");
        set_color!(LIGHT_GRAY);
        
        let mut idt = Idt::new();
        println!("    Divide-by-zero");
        idt.set_handler(0, handler!(divide_by_zero_handler));

        println!("    Debug");
        idt.set_handler(1, handler!(debug_exception_handler));

        println!("    Breakpoint");
        idt.set_handler(3, handler!(breakpoint_handler));

        println!("    Overflow");
        idt.set_handler(4, handler!(overflow_handler));

        println!("    Out-of-Bounds");
        idt.set_handler(5, handler!(out_of_bounds_handler));
        
        println!("    Invalid Instruction");
        idt.set_handler(6, handler!(invalid_opcode_handler));

        println!("    Device not available");
        idt.set_handler(7, handler!(device_not_available_handler));

        println!("    Double Fault");
        idt.set_handler(8, handler_with_raw_error!(double_fault_handler));

        println!("    Missing Segment");
        idt.set_handler(11, handler_with_error!(missing_segment_handler));

        println!("    Stack Fault");
        idt.set_handler(12, handler_with_error!(stack_fault_handler));

        println!("    General Protection Exception");
        idt.set_handler(13, handler_with_error!(general_protection_fault_handler));
        
        println!("    Page Fault");
        idt.set_handler(14, handler_with_raw_error!(page_fault_handler));
        
        println!("    Floating-Point Error");
        idt.set_handler(16, handler!(floating_point_error_handler));

        println!("    Alignment Check");
        idt.set_handler(17, handler_with_error!(alignment_check_handler));

        println!("    Machine Check");
        idt.set_handler(18, handler!(machine_check_handler));

        println!("    SIMD Error");
        idt.set_handler(19, handler!(simd_handler));

        println!("    Virtualization");
        idt.set_handler(20, handler!(virtualization_error_handler));

        log.ok();

        idt
    };
}


