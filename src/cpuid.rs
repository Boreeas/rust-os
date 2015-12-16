use core::fmt::Display;

#[derive(Debug)]
pub enum Vendor {
    OLD_AMD, // "AMDisbetter!"
    AMD,     // "AuthenticAMD"
    INTEL,   // "GenuineIntel"
    OLD_TRANSMETA, // "TransmetaCPU"
    TRANSMETA,     // "GenuineTMx86"
    CYRIX,   // "CyrixInstead"
    CENTAUR, // "CentaurHauls"
    NEXGEN,  // "NexGenDriven"
    UMC,     // "UMC UMC UMC "
    SIS,     // "SiS SiS SiS "
    NSC,     // "Geode by NSC"
    RISE,    // "RiseRiseRise"
}

impl Vendor {
    fn for_name(name: &[u8;12]) -> Vendor {
        use self::Vendor::*;

        match name {
            b"AMDisbetter!" => OLD_AMD,
            b"AuthenticAMD" => AMD,
            b"GenuineIntel" => INTEL,
            b"CentaurHauls" => CENTAUR,
            b"TransmetaCPU" => OLD_TRANSMETA,
            b"GenuineTMx86" => TRANSMETA,
            b"CyrixInstead" => CYRIX,
            b"NexGenDriven" => NEXGEN,
            b"UMC UMC UMC " => UMC,
            b"SiS SiS SiS " => SIS,
            b"Geode by NSC" => NSC,
            b"RiseRiseRise" => RISE,
            vendor         => panic!("Unknown CPU Vendor: {:?}", vendor)
        }
    }
}


pub fn get_vendor() -> Vendor {
    let CpuIdResult {
        eax: _,
        ebx: ebx,
        ecx: ecx,
        edx: edx
    } = cpuid(0);

    let buf = [
        ebx as u8, (ebx >> 8) as u8, (ebx >> 16) as u8, (ebx >> 24) as u8,
        edx as u8, (edx >> 8) as u8, (edx >> 16) as u8, (edx >> 24) as u8,
        ecx as u8, (ecx >> 8) as u8, (ecx >> 16) as u8, (ecx >> 24) as u8               
    ];

    Vendor::for_name(&buf)
    //loop{}
    //f
}


#[repr(C, packed)]
struct CpuIdResult {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32
}

fn cpuid(id: u32) -> CpuIdResult {
    let mut result = CpuIdResult {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
    };

    unsafe {
        internal_cpuid(id, &mut result);
    }

    result
}


#[link(name = "cpuid")]
extern {
    pub fn internal_cpuid(code: u32, ptr: *mut CpuIdResult);
}