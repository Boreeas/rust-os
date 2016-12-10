#[repr(C, packed)]
struct CpuIdResult {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
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
extern "C" {
    fn internal_cpuid(code: u32, ptr: *mut CpuIdResult);
}









#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Vendor {
    OLD_AMD, // "AMDisbetter!"
    AMD, // "AuthenticAMD"
    INTEL, // "GenuineIntel"
    OLD_TRANSMETA, // "TransmetaCPU"
    TRANSMETA, // "GenuineTMx86"
    CYRIX, // "CyrixInstead"
    CENTAUR, // "CentaurHauls"
    NEXGEN, // "NexGenDriven"
    UMC, // "UMC UMC UMC "
    SIS, // "SiS SiS SiS "
    NSC, // "Geode by NSC"
    RISE, // "RiseRiseRise"
}

impl Vendor {
    fn for_name(name: &[u8; 12]) -> Vendor {
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
            vendor => panic!("Unknown CPU Vendor: {:?}", vendor),
        }
    }
}


pub fn get_vendor() -> Vendor {
    let CpuIdResult {
        eax: _,
        ebx,
        ecx,
        edx
    } = cpuid(0);

    let buf = [ebx as u8,
               (ebx >> 8) as u8,
               (ebx >> 16) as u8,
               (ebx >> 24) as u8,
               edx as u8,
               (edx >> 8) as u8,
               (edx >> 16) as u8,
               (edx >> 24) as u8,
               ecx as u8,
               (ecx >> 8) as u8,
               (ecx >> 16) as u8,
               (ecx >> 24) as u8];

    Vendor::for_name(&buf)
}









bitflags! {
    flags Features: u64 {
        const FPU_PRESENT               = 1 <<  0,
        const VIRT_MODE_EXTENSIONS      = 1 <<  1,
        const DEBUGGING_EXTENSIONS      = 1 <<  2,
        const PAGE_SIZE_EXTENSIONS      = 1 <<  3,
        const TIMESTAMP_COUNTER         = 1 <<  4,
        const MODEL_SPECIFIC_REGISTERS  = 1 <<  5,
        const PHYSICAL_ADDR_EXTENSIONS  = 1 <<  6,
        const MACHINE_CHECK_EXCEPTION   = 1 <<  7,
        const COMPARE_AND_SWAP_8        = 1 <<  8,
        const APIC                      = 1 <<  9,
        const SYSENTER                  = 1 << 11,
        const MEMORY_TYPE_RANGE_REGS    = 1 << 12,
        const PAGE_GLOBAL_ENABLE        = 1 << 13,
        const MACHINE_CHECK_ARCHITECTURE= 1 << 14,
        const CONDITIONAL_MOVE          = 1 << 15,
        const PAGE_ATTRIBUTE_TABLE      = 1 << 16,
        const PAGE_SIZE_EXT_36_BITS     = 1 << 17,
        const PROCESSOR_SERIAL_NUMBER   = 1 << 18,
        const SSE2_CLFLUSH              = 1 << 19,
        const DBUG_STORE                = 1 << 21,
        const APIC_THERMAL_CONTROL      = 1 << 22,
        const MMX_INSTRUCTIONS          = 1 << 23,
        const FXSAVE_INSTRUCTIONS       = 1 << 24,
        const SSE_INSTRUCTIONS          = 1 << 25,
        const SSE2_INSTRUCTIONS         = 1 << 26,
        const CACHE_SELF_SNOOP          = 1 << 27,
        const HYPERTHREADING            = 1 << 28,
        const THERMAL_LIMITING          = 1 << 29,
        const IA64_EMULATING_X86        = 1 << 30,
        const PENDING_BREAK_ENABLE      = 1 << 31,
        // Ecx flags begin here           
        const SSE3_INSTRUCTIONS         = 1 << 32,
        const PCLMULQDQ                 = 1 << 33,
        const DEBUG_STORE_64BIT         = 1 << 34,
        const MONITOR_INSTRUCTIONS      = 1 << 35,
        const CPL_QUALIFIED_DEBUG_STORE = 1 << 36,
        const VIRT_MACHINE_EXTENSIONS   = 1 << 37,
        const SAFER_MODE_EXTENSIONS     = 1 << 38,
        const ENHANCED_SPEED_STEP       = 1 << 39,
        const THERMAL_MONITOR_2         = 1 << 40,
        const SUPPLEMENTAL_SSE3         = 1 << 41,
        const L1_CONTEXT_ID             = 1 << 42,
        const SILICON_DEBUG_INTERFACE   = 1 << 43,
        const FUSED_MULTIPLY_ADD        = 1 << 44,
        const COMPARE_AND_SWAP_16       = 1 << 45,
        const DISABLE_TASK_PRIORITY_MSG = 1 << 46,
        const PERFMON_AND_DEBUG_CAPAB   = 1 << 47,
        const PROCESS_CONTEXT_IDS       = 1 << 49,
        const DMA_DIRECT_CACHE_ACCESS   = 1 << 50,
        const SSE41                     = 1 << 51,
        const SSE42                     = 1 << 52,
        const X2APIC                    = 1 << 53,
        const MOVE_BIGENDIAN            = 1 << 54,
        const POPCNT                    = 1 << 55,
        const TSC_DEADLINE              = 1 << 56,
        const AES_INSTRUCTIONS          = 1 << 57,
        const XSAVE_INSTRUCTIONS        = 1 << 58,
        const OS_XSAVE_ENABLED          = 1 << 59,
        const ADV_VECTOR_EXTENSIONS     = 1 << 60,
        const HALF_PRECISION_FP         = 1 << 61,
        const RDRAND                    = 1 << 62,
        const IS_HYPERVISOR             = 1 << 63
    }
}

pub fn get_features() -> Features {
    let CpuIdResult {
        eax: _,
        ebx: _,
        ecx,
        edx
    } = cpuid(1);

    Features::from_bits((ecx as u64) << 32 | (edx as u64)).unwrap()
}
