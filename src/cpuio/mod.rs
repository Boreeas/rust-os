mod port;

use core::ptr::Unique;
use spin::Mutex;
pub use self::port::{Port, UnsafePort};
use memory::FrameAllocator;
use memory::paging;
use cpuid;
use cpuid::get_features;

#[link(name = "interrupts")]
extern {
    fn interrupt_handler();
}

pub const APIC_ADDRESS_BASE: usize = 0xfee00000;
pub const APIC: Mutex<Apic> = Mutex::new(Apic {
    registers: unsafe { Unique::new(APIC_ADDRESS_BASE as *mut _) }
});

const LOCAL_APIC_ID: usize                      = 0x02;
const END_OF_INTERRUPT: usize                   = 0x0b;
const SPURIOUS_INTERRUPT_VECTOR_REGISTER: usize = 0x0f;

#[repr(C,packed)]
struct Register {
    _unused: [u32; 3],
    value: u32
}

pub struct Apic {
    registers: Unique<[Register; 16]>
}

impl Apic {
    fn get_local_apic_id(&self) -> u32 {
        unsafe { self.registers.get()[LOCAL_APIC_ID].value }
    }

    fn get_spurious_interrupt_vector(&self) -> u32 {
        unsafe {
            self.registers.get()[SPURIOUS_INTERRUPT_VECTOR_REGISTER].value
        }
    }

    fn set_spurious_interrupt_vector(&mut self, value: u32) {
        unsafe {
            self.registers.get_mut()[SPURIOUS_INTERRUPT_VECTOR_REGISTER].value = value;
        }
    }
}




pub fn setup_apic<A>(alloc: &mut A) where A: FrameAllocator {
    if !get_features().contains(cpuid::APIC) {
        panic!("No APIC support found");
    }

    let log = log!("Setting up APIC");
    disable_8259_pic();
    map_apic_registers(alloc);
    setup_idt(alloc);
    enable_interrupts();


    log.ok();
}

fn disable_8259_pic() {
    let log = log!("  Disabling 8259 PIC");

    unsafe { 
        let pic1_cmd  = UnsafePort::<u8>::new(0x20);
        let pic1_data = UnsafePort::<u8>::new(0x21);
        let pic2_cmd  = UnsafePort::<u8>::new(0xa0);
        let pic2_data = UnsafePort::<u8>::new(0xa1);
        
        // ICW1
        pic1_cmd.write(0x11);
        pic2_cmd.write(0x11);

        // IRQ base offset
        pic1_data.write(0xe0);
        pic2_data.write(0xe8);

        // ICW3
        pic1_data.write(0x04);
        pic2_data.write(0x02);

        // ICW4
        pic1_data.write(0x01);
        pic2_data.write(0x01);

        // Interrupt masks
        pic1_data.write(0xff);
        pic2_data.write(0xff);
    }

    log.ok();
}

fn map_apic_registers<A>(alloc: &mut A) where A: FrameAllocator {
    let log = log!("  Identity-mapping APIC registers");

    paging::simple_id_map(paging::Frame::for_address(APIC_ADDRESS_BASE), alloc);

    log.ok();
}

fn enable_interrupts() {
    let log = log!("  Enabling interrupts");

    let _apic = APIC;
    let mut apic = _apic.lock();
    let siv = apic.get_spurious_interrupt_vector();
    // Interrupt enable bit, spurious interrupt number
    apic.set_spurious_interrupt_vector(siv | 1 << 8 | 0xff);

    log.ok();
}

fn setup_idt<A>(alloc: &mut A) where A: FrameAllocator {
    let log = log!("  Creating IDT");

    let idt_buf = paging::alloc_any(alloc);
    for i in 0..256 {
        let entry = IdtEntry {
            offset: unsafe { &interrupt_handler as *const _ as u64 },
            present: false,
            dpl: 0,
            is_storage_segment: false,
            gate_type: GateType::INTERRUPT_GATE_32,
            selector: 0
        };

        write_idt_entry(idt_buf, entry, i);
    }

    let idt_info = [0u8; 10];
    unsafe {
       let ptr = &idt_info as *const _;
        *(ptr as *mut u16) = 4095;
        *((ptr as usize + 2) as *mut u64) = idt_buf as *const _ as u64;

        asm!("lidt %0" :: "p"(ptr) :: "volatile")
    }


    log.ok();
}

fn write_idt_entry(idt: &mut [u8; 4096], entry: IdtEntry, idx: usize) {
    assert!(idx < 256);

    let offset = idx *  16;
    unsafe {
        let ptr = (idt as *mut _ as usize) + offset;
        *(ptr      as *mut u16) = entry.offset_low_bits();
        *((ptr+ 2) as *mut u16) = entry.selector;
        *((ptr+ 4) as *mut  u8) = 0;
        *((ptr+ 5) as *mut  u8) = entry.type_and_attr();
        *((ptr+ 6) as *mut u16) = entry.offset_middle_bits();
        *((ptr+ 8) as *mut u32) = entry.offset_high_bits();
        *((ptr+12) as *mut u32) = 0;
    }
}

struct IdtEntry {
    offset: u64,
    present: bool,
    dpl: u8,
    is_storage_segment: bool,
    gate_type: GateType,
    selector: u16,
}

impl IdtEntry {
    fn offset_low_bits(&self) -> u16 {
        (self.offset & 0xffff) as u16
    }

    fn offset_middle_bits(&self) -> u16 {
        ((self.offset >> 16) & 0xffff) as u16
    }

    fn offset_high_bits(&self) -> u32 {
        ((self.offset >> 32) & 0xffffffff) as u32
    }

    fn type_and_attr(&self) -> u8 {
        self.gate_type as u8
        | if self.is_storage_segment { 1 << 4 } else { 0 }
        | self.dpl & 0b11 << 5
        | if self.present { 1 << 7 } else { 0 }
    }
}

#[derive(Copy,Clone)]
enum GateType {
    TASK_GATE_32 =          0x05,
    INTERRUPT_GATE_16 =     0x06,
    TRAP_GATE_16 =          0x07,
    INTERRUPT_GATE_32 =     0x0e,
    TRAP_GATE_32 =          0x0f
}