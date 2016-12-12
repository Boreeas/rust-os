mod exceptions;

use spin::Mutex;
use self::exceptions::{ExceptionStackFrame, PageFaultErrorCode};

pub type HandlerFunc = extern "C" fn() -> !;
const NUM_ENTRIES: usize = 256;

#[repr(C,packed)]
#[derive(Clone, Copy)]
// Length: 16 bytes
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    _unused1: u8,
    attributes: u8,
    offset_middle: u16,
    offset_high: u32,
    _unused2: u32,
}

impl IdtEntry {
    const fn not_set() -> IdtEntry {
        IdtEntry {
            offset_low: 0,
            offset_middle: 0,
            offset_high: 0,
            selector: 0,
            attributes: 0,
            _unused1: 0,
            _unused2: 0
        }
    }

    fn new(ptr: HandlerFunc, attributes: Attributes) -> IdtEntry {
        let addr = ptr as usize;
        IdtEntry {
            offset_low: addr as u16,
            offset_middle: (addr >> 16) as u16,
            offset_high: (addr >> 32) as u32,
            selector: 0x08,
            _unused1: 0,
            attributes: attributes.bits,
            _unused2: 0 
        }
    }
}

bitflags! {
    flags Attributes: u8 {
        const TASK_GATE_32      = 0b0101,
        const INTERRUPT_GATE_16 = 0b0110,
        const TRAP_GATE_16      = 0b0111,
        const INTERRUPT_GATE_32 = 0b1110,
        const TRAP_GATE_32      = 0b1111,
        
        const STORAGE_SEGMENT   = 1 << 4,
        
        const DPL_0             = 0 << 5,
        const DPL_1             = 1 << 5,
        const DPL_2             = 2 << 5,
        const DPL_3             = 3 << 5,

        const PRESENT           = 1 << 7,
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

macro_rules! handler_with_error {
    ($name:ident) => ({
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_scratch_registers!();

                asm!("mov rsi, [rsp+9*8]
                      mov rdi, rsp
                      add rdi, 10*8
                      call $0"
                      :: "i"($name as extern "C" fn(&ExceptionStackFrame, u64))
                      : "rdi", "rsi" : "intel");

                restore_scratch_registers!();
                asm!("add rsp, 8
                      iretq"
                      :::: "intel", "volatile");

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

    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) {

        self.0[entry as usize] = IdtEntry::new(handler, INTERRUPT_GATE_32 | PRESENT | DPL_0)
    }

    pub fn set_entry(&mut self, entry: u8, handler: IdtEntry) {
        self.0[entry as usize] = handler;
    }
}

lazy_static! {
    pub static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(3, handler!(breakpoint_handler));
        idt.set_handler(6, handler!(invalid_opcode_handler));
        idt.set_handler(14, handler_with_error!(page_fault_handler));
        idt
    };
}


extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) {
    use ::vga_buffer::Color::*;

    set_color!(RED);
    print!("\nERROR: ");
    set_color!(WHITE);
    println!("division by zero");
    set_color!(LIGHT_GRAY);
    println!("{:#?}", stack_frame);
}

extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    use ::vga_buffer::Color::*;

    set_color!(RED);
    print!("\nERROR: ");
    set_color!(WHITE);
    println!("invalid opcode at {:#x}", stack_frame.instruction_pointer);
    set_color!(LIGHT_GRAY);
    println!("{:#?}", stack_frame);
}

extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, errno: u64) {
    use ::vga_buffer::Color::*;
    use ::x86::shared::control_regs;

    set_color!(RED);
    print!("\nERROR: ");
    set_color!(WHITE);
    println!("page fault trying to access 0x{:x} ({:?})", 
        unsafe { control_regs::cr2() },
        PageFaultErrorCode::from_bits(errno).unwrap());
    set_color!(LIGHT_GRAY);
    println!("{:#?}", stack_frame);
}

extern "C" fn breakpoint_handler(stack_frame: &ExceptionStackFrame) {
    use ::vga_buffer::Color::*;
    
    set_color!(RED);
    print!("\nBREAKPOINT: ");
    set_color!(WHITE);
    println!("At instruction {:#x}", stack_frame.instruction_pointer);
    set_color!(LIGHT_GRAY);
    println!("{:#?}", stack_frame);
}
