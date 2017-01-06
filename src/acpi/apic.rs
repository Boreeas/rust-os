use super::{AcpiSdtHeader};
use core::mem;

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct ApicSdt {
    pub header: AcpiSdtHeader,
    pub local_controller_addr: u32,
    pub flags: u32,
    pub first_entry: u32
}

impl ApicSdt {
    pub fn iter(&'static self) -> ApicSdtIterator {
        ApicSdtIterator::new(self)
    }
}

pub struct ApicSdtIterator {
    table: &'static ApicSdt,
    offset: u32
}

impl ApicSdtIterator {
    fn new(table: &'static ApicSdt) -> ApicSdtIterator {
        ApicSdtIterator {
            table: table,
            offset: mem::size_of::<ApicSdt>() as u32 - 4
        }
    }
}

impl Iterator for ApicSdtIterator {
    type Item = EntryType;

    fn next(&mut self) -> Option<EntryType> {
        if self.offset >= self.table.header.length {
            return None
        }

        let entry_addr = self.table as *const _ as u32 + self.offset;

        let entrytype = unsafe { *(entry_addr as *const u8) };
        let result = match entrytype {
            0 => EntryType::ProcessorApic( unsafe { 
                &*((entry_addr + 2) as *const _)
            }),
            1 => EntryType::IoApic( unsafe {
                &*((entry_addr + 2) as *const _)
            }),
            2 => EntryType::InterruptOverride( unsafe {
                &*((entry_addr + 2) as *const _)
            }),
            3 => EntryType::NonMaskableInterruptSource( unsafe {
                &*((entry_addr + 2) as *const _)
            }),
            4 => EntryType::ProcessorApicNmi( unsafe {
                &*((entry_addr + 2) as *const _)
            }),
            x => EntryType::Unknown(x),
        };

        let length = unsafe { *((entry_addr + 1) as *const u8) as u32 };
        self.offset += length;
        Some(result)
    } 
}

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct ProcessorApicInfo {
    pub proc_id: u8,
    pub apic_id: u8,
    pub flags: u32
}

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct IoApicInfo {
    pub apic_id: u8,
    _reserved: u8,
    pub address: u32,
    pub global_irq_base: u32    
}

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct InterruptOverrideInfo {
    pub bus: u8,
    pub source_irq_base: u8,
    pub global_irq_base: u32,
    pub flags: u16
}

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct NmiSourceInfo {
    pub flags: u16,
    pub global_irq_base: u32
}

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct ProcessorApicNmiInfo {
    pub proc_id: u8,
    pub flags: u16,
    pub lint: u8 // linux doc says LINTn - local interupt?
}



#[derive(Debug)]
pub enum EntryType {
    ProcessorApic(&'static ProcessorApicInfo),
    IoApic(&'static IoApicInfo),
    InterruptOverride(&'static InterruptOverrideInfo),
    NonMaskableInterruptSource(&'static NmiSourceInfo),
    ProcessorApicNmi(&'static ProcessorApicNmiInfo),
    Unknown(u8)
}