use super::{AcpiSdtHeader, SdtError, AcpiSdt};

use core::mem;

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct AcpiRootSdt {
    pub header: AcpiSdtHeader,
    pub first_entry: u32
}

impl AcpiRootSdt {
    pub fn num_entries(&self) -> u32 {
        (self.header.length - mem::size_of::<AcpiSdtHeader>() as u32) / 4
    }

    pub fn iter(&'static self) -> AcpiRootSdtIterator {
        AcpiRootSdtIterator {
            current: 0,
            table: self
        }
    }
}

pub struct AcpiRootSdtIterator {
    current: u32,
    table: &'static AcpiRootSdt
}

impl Iterator for AcpiRootSdtIterator {
    type Item = Result<AcpiSdt, SdtError>;

    fn next(&mut self) -> Option<Result<AcpiSdt, SdtError>> {
        if self.current >= self.table.num_entries() {
            return None;
        }

        let offset = 4*self.current; // 4 byte ptrs
        let addr_of_next_entry = &self.table.first_entry as *const _ as u32 + offset;
        let entry = unsafe { 
            *(addr_of_next_entry as *const u32) as *const AcpiSdtHeader
        };
        self.current += 1;
        return Some(unsafe { (&*entry).to_full_table() })
    }
}
