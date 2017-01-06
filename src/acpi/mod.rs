pub mod root;
pub mod apic;

use core::slice;
use core::mem;
use core::str;

use self::root::AcpiRootSdt;
use self::apic::ApicSdt;


#[derive(Debug,Copy,Clone)]
pub enum RsdpError {
    NotFound,
    ValidationFailed
}

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct RsdpV1 {
    pub signature: [u8; 8], // technically 7-bit ascii
    pub checksum: u8,
    pub oem_id: [u8; 6], // ascii, too
    pub revision: u8,
    pub address: *const AcpiRootSdt
}

impl RsdpV1 {
    fn validate(&self) -> bool {
        let base = self as *const _ as usize;
        let mut offset = 0;
        let mut sum: u8 = 0;

        while offset < mem::size_of::<RsdpV1>() {
            unsafe {
                sum = sum.wrapping_add(*((base + offset) as *const u8));
                offset += 1;
            }
        }

        sum == 0
    }
}

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct RsdpV2 {
    pub signature: [u8; 8], // technically 7-bit ascii
    pub checksum: u8,
    pub oem_id: [u8; 6], // ascii, too
    pub revision: u8,
    pub short_address: *const AcpiRootSdt,
    pub length: u32,
    pub long_address: *const u8,
    pub extended_checksum: u8,
    pub reserved: [u8; 3]
}

impl RsdpV2 {
    fn validate(&self) -> bool {
        let base = self as *const _ as usize;
        let mut offset = 0;
        let mut sum: u8 = 0;

        // First part
        while offset < mem::size_of::<RsdpV1>() {
            unsafe {
                sum = sum.wrapping_add(*((base + offset) as *const u8));
                offset += 1;
            }
        }

        // Second part
        let mut ext_sum: u8 = 0;
        while offset < mem::size_of::<RsdpV2>() {
            unsafe {
                ext_sum = sum.wrapping_add(*((base + offset) as *const u8));
                offset += 1;
            }
        }

        sum == 0 && ext_sum == 0
    }
}

#[derive(Debug,Copy,Clone)]
pub enum Rsdp {
    V1(&'static RsdpV1),
    V2(&'static RsdpV2)
}

impl Rsdp {
    fn locate() -> Option<*const RsdpV1> {
        let mut rsdp_offset = 0xe0000;
        while rsdp_offset < 0xfffff {
            let buf = unsafe { 
                slice::from_raw_parts(rsdp_offset as *const u8, 8)
            };

            if buf == b"RSD PTR " {
                return Some(rsdp_offset as *const RsdpV1)
            }

            rsdp_offset += 2;
        }

        None
    }

    pub fn try_load() -> Result<Rsdp, RsdpError> {
        if let Some(address) = Rsdp::locate() {
            let rsdp = unsafe { &*address };
            if rsdp.revision == 0 {
                if rsdp.validate() {
                    return Ok(Rsdp::V1(rsdp))
                }
            } else {
                let rsdp: &'static RsdpV2 = unsafe { &*(address as *const RsdpV2) };
                if rsdp.validate() {
                    return Ok(Rsdp::V2(rsdp))
                }
            }

            return Err(RsdpError::ValidationFailed)
        }

        Err(RsdpError::NotFound)
    }
}

#[repr(C,packed)]
#[derive(Debug,Copy,Clone)]
pub struct AcpiSdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32
}

impl AcpiSdtHeader {
    pub fn to_full_table(&self) -> Result<AcpiSdt, SdtError> {
        
        if !self.validate() {
            return Err(SdtError::ValidationFailed)
        }

        match &self.signature {
            //b"FACP" => panic!("unimplemented"),
            //b"SSDT" => panic!("unimplemented"),
            b"APIC" => {
                let ptr = self as *const _ as *const ApicSdt;
                Ok(AcpiSdt::Apic(unsafe { &*ptr }))
            },
            b"RSDT" => {
                let ptr = self as *const _ as *const AcpiRootSdt;
                Ok(AcpiSdt::Root(unsafe { &*ptr }))
            }
            _ => Err(SdtError::UnknownType)
        }
    }

    pub fn signature_as_str(&self) -> &str {
        return str::from_utf8(&self.signature).ok().expect("Invalid Utf8 in Acpi header")
    }

    pub fn validate(&self) -> bool {
        let base = self as *const _ as u32;
        let mut sum: u8 = 0;

        for offset in 0..self.length {
            unsafe {
                sum = sum.wrapping_add(*((base + offset) as *const u8));
            }
        }

        sum == 0
    }
}

#[derive(Debug,Copy,Clone)]
pub enum SdtError {
    UnknownType,
    ValidationFailed
}

#[derive(Debug,Copy,Clone)]
pub enum AcpiSdt {
    Root(&'static AcpiRootSdt),
    Apic(&'static ApicSdt)
}