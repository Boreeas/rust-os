use spin::Mutex;

pub type HandlerFunc = extern "C" fn() -> !;
const NUM_ENTRIES: u16 = 256;

#[repr(C,packed)]
#[derive(Clone, Copy)]
// Length: 16 bytes
struct IdtEntry {
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
            selector: 0,
            _unused1: 0,
            attributes: 0,
            offset_middle: 0,
            offset_high: 0,
            _unused2: 0
        }
    }

    fn new(ptr: unsafe fn(), attributes: Attributes) -> IdtEntry {
        let addr = &ptr as *const _ as usize;
        IdtEntry {
            offset_low: (addr & 0xffff) as u16,
            selector: 0x08,
            _unused1: 0,
            attributes: attributes.bits,
            offset_middle: (addr >> 16 & 0xffff) as u16,
            offset_high: (addr >> 32 & 0xffffffff) as u32,
            _unused2: 0 
        }
    }
}

bitflags! {
    flags Attributes: u8 {
        const TASK_GATE_32      = 0x05,
        const INTERRUPT_GATE_16 = 0x06,
        const TRAP_GATE_16      = 0x07,
        const INTERRUPT_GATE_32 = 0x0e,
        const TRAP_GATE_32      = 0x0f,
        
        const STORAGE_SEGMENT   = 1 << 4,
        
        const ACCESS_RING_0     = 0 << 5,
        const ACCESS_RING_1     = 1 << 5,
        const ACCESS_RING_2     = 2 << 5,
        const ACCESS_RING_3     = 3 << 5,

        const PRESENT           = 1 << 7,
    }
}

macro_rules! gen_interrupt_handler {
    ($name:ident: $handler:ident) => (

        #[naked]
        unsafe fn $name() {
            asm! {
                "push rbp
                  push r15
                  push r14
                  push r13
                  push r12
                  push r11
                  push r10
                  push r9
                  push r8
                  push rsi
                  push rdi
                  push rdx
                  push rcx
                  push rbx
                  push rax
                  mov rsi, rsp
                  push rsi

                  cli
                  call $0
                  sti
                  
                  add rsp, 8
                  pop rax
                  pop rbx
                  pop rcx
                  pop rdx
                  pop rdi
                  pop rsi
                  pop r8
                  pop r9
                  pop r10
                  pop r11
                  pop r12
                  pop r13
                  pop r14
                  pop r15
                  pop rbp
                  iretq" :: "s"($handler as fn()) :: "volatile", "intel"
            }
        }
    )
}

macro_rules! idt_entry {
    (task32 $name:ident: $body:expr) => ({
        gen_interrupt_handler!($name: $body);

        IdtEntry::new(
            $name, 
            TASK_GATE_32 | STORAGE_SEGMENT | PRESENT
        )
    });

    (trap32 $name:ident: $body:expr) => ({
        gen_interrupt_handler!($name: $body);

        IdtEntry::new(
            $name,
            TRAP_GATE_32 | PRESENT
        )
    });

    (int32 $name:ident: $body:expr) => ({
        gen_interrupt_handler!($name: $body);
        
        IdtEntry::new(
            $name,
            INTERRUPT_GATE_32 | PRESENT
        )
    });

    (ring3 int32 $name:ident: $body:expr) => ({
        gen_interrupt_handler!($name: $body);
        
        IdtEntry::new(
            $name,
            INTERRUPT_GATE_32 | PRESENT | ACCESS_RING_3
        )
    });
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

    fn load(&'static self) {
        let idtinfo = IdtRef {
            limit: 20 * NUM_ENTRIES,
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

    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc)
        -> &mut Attributes {

        self.0[entry] = idt_entry!(int32 inthandler: handler);
    }

    pub fn set_entry(&mut self, entry: u8, handler: IdtEntry)
        -> &mut Attributes {

        self.0[entry] = handler
    }
}

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.set_handler(0, divide_by_zero_handler);
        idt
    };
}