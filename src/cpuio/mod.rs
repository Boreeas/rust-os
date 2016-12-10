mod port;
mod interrupts;

use core::ptr::Unique;
use spin::Mutex;
pub use self::port::{Port, UnsafePort};
use memory::FrameAllocator;
use memory::paging;
use cpuid;
use cpuid::get_features;

pub const APIC_ADDRESS_BASE: usize = 0xfee00000;
pub const APIC: Mutex<Apic> = Mutex::new(Apic {
    registers: unsafe { Unique::new(APIC_ADDRESS_BASE as *mut _) },
});

const LOCAL_APIC_ID: usize = 0x02;
const END_OF_INTERRUPT: usize = 0x0b;
const SPURIOUS_INTERRUPT_VECTOR_REGISTER: usize = 0x0f;

#[repr(C,packed)]
struct Register {
    _unused: [u32; 3],
    value: u32,
}

pub struct Apic {
    registers: Unique<[Register; 16]>,
}

impl Apic {
    fn get_local_apic_id(&self) -> u32 {
        unsafe { self.registers.get()[LOCAL_APIC_ID].value }
    }

    fn get_spurious_interrupt_vector(&self) -> u32 {
        unsafe { self.registers.get()[SPURIOUS_INTERRUPT_VECTOR_REGISTER].value }
    }

    fn set_spurious_interrupt_vector(&mut self, value: u32) {
        unsafe {
            self.registers.get_mut()[SPURIOUS_INTERRUPT_VECTOR_REGISTER].value = value;
        }
    }
}




pub fn setup_apic<A>(alloc: &mut A)
    where A: FrameAllocator
{
    if !get_features().contains(cpuid::APIC) {
        panic!("No APIC support found");
    }

    let log = log!("Setting up APIC");
    disable_8259_pic();
    map_apic_registers(alloc);
    interrupts::init_idt();
    enable_interrupts();


    log.ok();
}

fn disable_8259_pic() {
    let log = log!("  Disabling 8259 PIC");

    unsafe {
        let pic1_cmd = UnsafePort::<u8>::new(0x20);
        let pic1_data = UnsafePort::<u8>::new(0x21);
        let pic2_cmd = UnsafePort::<u8>::new(0xa0);
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

fn map_apic_registers<A>(alloc: &mut A)
    where A: FrameAllocator
{
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