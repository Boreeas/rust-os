fn get_cr0() -> u64 {
    let reg: u64;
    unsafe { 
        asm!("mov $0, cr0" : "=r"(reg) ::: "intel", "volatile");
    }

    reg
}

fn set_cr0(value: u64) {
    unsafe {
        asm!("mov cr0, $0" :: "r"(value) : "cr0", "memory" : "intel", "volatile");
    }
}

fn get_cr3() -> u64 {
    let reg: u64;
    unsafe { 
        asm!("mov $0, cr3" : "=r"(reg) ::: "intel", "volatile");
    }

    reg
}

fn set_cr3(value: u64) {
    unsafe {
        asm!("mov cr3, $0" :: "r"(value) : "cr3", "memory" : "intel", "volatile");
    }
}

fn get_cr4() -> u64 {
    let reg: u64;
    unsafe { 
        asm!("mov $0, cr4" : "=r"(reg) ::: "intel", "volatile");
    }

    reg
}

fn set_cr4(value: u64) {
    unsafe {
        asm!("mov cr4, $0" :: "r"(value) : "cr4", "memory" : "intel", "volatile");
    }
}

fn get_cr8() -> u64 {
    let reg: u64;
    unsafe { 
        asm!("mov $0, cr8" : "=r"(reg) ::: "intel", "volatile");
    }

    reg
}

fn set_cr8(value: u64) {
    unsafe {
        asm!("mov cr8, $0" :: "r"(value) : "cr8", "memory" : "intel", "volatile");
    }
}

fn get_efer() -> u64 {
    let high: u32;
    let low: u32;
    unsafe {
        asm!("rdmsr" 
            : "={eax}"(low), "={edx}"(high)
            : "{ecx}"(0xC0000080u32)
            :: "intel", "volatile")
    }

    (high as u64) << 32 | (low as u64)
}

fn set_efer(value: u64) {
    let high = (value >> 32) as u32;
    let low = value as u32;
    unsafe {
        asm!("wrmsr"
            :: "{ecx}"(0xC0000080u32), "{eax}"(low), "{edx}"(high)
            :: "intel", "volatile")
    }
}

fn read_rflags() -> u64 {
    let result: u64;

    unsafe {
        asm!("pushfq
              pop $0" 
              : "=r"(result) 
              :: "memory" 
              : "intel", "volatile")
    }

    // Second bit is always 1
    result & !2
}

fn write_rflags(value: u64) {
    unsafe {
        asm!("push $0
              popfq"
              :: "r"(value | 2) // second bit is always 1
              : "memory", "cc"
              : "intel", "volatile")
    }
}



pub mod efer {
    bitflags! {
        // Bits in the Extended Feature Enable Register
        flags Efer: u64 {
            // Is the SYSCALL instruction enabled?
            const SYSCALL_ENABLED = 1 << 0,
            // Set to enable long mode
            const LONG_MODE_ENABLED = 1 << 8,
            // Indicates if long mode is enabled (read only)
            const IS_LONG_MODE_ENABLED = 1 << 10,
            // Is the EXECUTE_DISABLE bit in page table entries enabled? 
            const EXECUTE_DISABLE_BIT_ENABLED = 1 << 11,
        }
    }

    impl Efer {
        pub fn load() -> Efer {
            let bits = super::get_efer();
            Efer::from_bits(bits).expect("Unknown EFER")
        }

        pub fn store(&self) {
            super::set_efer(self.bits);
        }
    }
}

pub mod rflags {
    bitflags! {
        // Bits in the EFLAGS register
        flags RFlags: u64 {
            // Carry bit, set by some instructions to indicate overflow
            const CARRY = 1 << 0,
            // Indicates an even number of bits in the low byte of the
            // result of some instructions
            const PARITY = 1 << 2,
            // Indicates carry from BCD
            const AUXILLARY = 1 << 4,
            // Indicates that the result from the last arithm. instr. was 0
            const ZERO = 1 << 6,
            // Indicates that the result from the last airthm. instr. was <0
            const NEGATIVE = 1 << 7,
            // Set to enable single step debugging: Every 
            // instruction generates a #DB
            const SINGLE_STEP = 1 << 8,
            // Set to enable maskable hardware interrupts
            const HARDWARE_INTERRUPTS = 1 << 9,
            // IO privilege levels
            const IOPL_0 = 0 << 12,
            const IOPL_1 = 1 << 12,
            const IOPL_2 = 2 << 12,
            const IOPL_3 = 3 << 12,
            
            // Nested tasks (disabled in long mode)
            //const NESTED_TASKS = 1 << 14,
            
            // Disable instruction-breakpoint traps
            const INSTRUCTION_BREAKPOINTS_DISABLED = 1 << 16,
            
            // Enable 8086 virtual mode (disabled in longmode)
            // const VIRTUAL_MODE = 1 << 17,
            
            // Enable alignment checking (via CR0) or 
            // supervisor access control (via CR4)
            const ACCESS_CONTROL_ALIGNMENT_CHECK = 1 << 18,
            // Enable virtual interrupts
            const VIRTUAL_INTERRUPTS = 1 << 19,
            // Set to mark a virtual interrupt
            const VIRTUAL_INTERRUPT_PENDING = 1 << 20,
            // Test bit to check for CPUID. Set or clear to test
            const CPUID = 1 << 21
        }
    }

    impl RFlags {
        pub fn load() -> RFlags {
            let bits = super::read_rflags();

            RFlags::from_bits(bits).expect("Unknown RFlags")
        }

        pub fn store(&self) {
            super::write_rflags(self.bits);
        }

        pub fn get_io_privilege_level(&self) -> u8 {
            if *self & IOPL_3 == IOPL_3 {
                3
            } else if *self & IOPL_2 == IOPL_2 {
                2
            } else if *self & IOPL_1 == IOPL_1 {
                1
            } else {
                0
            }
        }
    }
}

pub mod cr0 {
    bitflags! {
        flags CR0: u64 {
            // Enable proctected mode
            const PROTECTED_MODE = 1 << 0,
            // Enable FPU monitoring. Raises NM# if (F)WAIT is executed and
            // FPU_STATE_SWITCHED is set
            const MONITOR_COPROCESSOR = 1 << 1,
            // Indicates that no internal or external FPU processor is present
            const FPU_MISSING = 1 << 2, 
            // Indicates that a task switch occured and the FPU/MMX/SSE state
            // is dirty
            const FPU_STATE_SWITCHED = 1 << 3, 
            // Indicate support of i387 math coprocessor instructions
            const DX_MATH_COPROCESSOR = 1 << 4,
            // Enable native fpu error reporting
            const NATIVE_FPU_ERRORS = 1 << 5,
            // Prevent writing into read-only pages by supervisor processes
            const WRITE_PROTECT = 1 << 16, 
            // Enable automatic alignment checking. Needs to be combined with
            // RFlags, only works at CPL 3
            const ALIGNMENT_CHECKING = 1 << 18,
            // Disable write-back and write-through cache strategies
            const NO_WRITE_BACK = 1 << 29,
            // Disables caching. Needs to be followed up with 
            // cache invalidation
            const CACHING_DISABLED = 1 << 30,
            // Enable paging
            const PAGING = 1 << 31,
        }
    }

    impl CR0 {
        pub fn load() -> CR0 {
            let bits = super::get_cr0();

            CR0::from_bits(bits).expect("Unknown CR0 flags")
        }

        pub fn store(&self) {
            super::set_cr0(self.bits);
        }
    }
}

pub mod cr3 {
    // P4 table address has lower 12 bits cleared (4096 bit aligned)
    const P4_TABLE_MASK: u64 = !0b1111_1111_1111;

    pub fn p4_table_address() -> u64 {
        return super::get_cr3() & P4_TABLE_MASK
    }

    pub fn set_p4_table_address(table: u64) {
        if table & !P4_TABLE_MASK > 0 {
            panic!("P4 table address {:#x} not 4096-bit aligned", table);
        }

        super::set_cr8(table | (super::get_cr8() & !P4_TABLE_MASK));
    }
}

pub mod cr4 {
    bitflags! {
        flags CR4: u64 {
            // Enable interrupts and exceptions in virtual mode
            const VIRTUAL_MODE_EXTENSIONS = 1 << 0,
            // Enable virtual interrupts in protected mode
            const PROTECTED_MODE_VIRT_INTERRUPTS = 1 << 1,
            // Restrict usage of RDTSC to CPL 0
            const RESTRICTED_TIMESTAMPS = 1 << 2,
            // Disable DR4 and DR5
            const DISABLE_DEBUG_REGISTERS = 1 << 3,
            // Enable huge pages
            const PAGE_SIZE_EXTENSION = 1 << 4,
            // Enable 64bit addresses
            const PHYSICAL_ADDRESS_EXTENSION = 1 << 5,
            // Enable machine check
            const MACHINE_CHECK = 1 << 6,
            // Enable global pages
            const GLOBAL_PAGES = 1 << 7,
            // Enable RDPMC for all access levels, otherwise restrict to CPL 0
            const PERFORMANCE_COUNTER = 1 << 8,
            // Enable FXSTOR, FXSAV and SSE instructions
            const SSE_ENABLED = 1 << 9,
            // Enable unmasked SIMD floating point instruction
            const UNMASKED_SIMD_EXCEPTIONS = 1 << 10,
            // Restrict SIDT/SGDT/SLDT/... instructions to CPL 0
            const RESTRICTED_DESCRIPTOR_TABLES = 1 << 11,
            // Enable virtual machine extensions
            const VIRTUAL_MACHINE_EXTENSISONS = 1 << 13,
            // Enable safer mode extensions
            const SAFER_MODE_EXTENSIONS = 1 << 14,
            // Enable FSGSBASE instructions
            const FSGSBASE = 1 << 16,
            // Enable process context identifiers
            const PCID = 1 << 17,
            // Enable XSAVE instructions
            const XSAVE = 1 << 18,
            // Prevent supervisor from executing instructions
            const SUPERVISOR_EXECUTION_PREVENTION = 1 << 20,
            // Prevent supervisor from accessing pages
            const SUPERVISOR_ACCESS_PREVENTION = 1 << 21,
            // Enable memory protection keys
            const MEMORY_PROTECTION_KEYS = 1 << 22,
        }
    }

    impl CR4 {
        pub fn load() -> CR4 {
            let bits = super::get_cr4();

            CR4::from_bits(bits).expect("Unknown CR4 flags")
        }

        pub fn store(&self) {
            super::set_cr4(self.bits);
        }
    }
}

pub mod cr8 {
    const TASK_PRIORITY_LEVEL_MASK: u64 = 0b1111;

    pub fn get_task_priority_level() -> u8 {
        return (super::get_cr8() & TASK_PRIORITY_LEVEL_MASK) as u8
    } 

    pub fn set_task_priority_level(tpl: u64) {
        if tpl > TASK_PRIORITY_LEVEL_MASK {
            panic!("Unsupported task protection level {:?}", tpl);
        }

        super::set_cr8(tpl | (super::get_cr8() & !TASK_PRIORITY_LEVEL_MASK));
    }
}