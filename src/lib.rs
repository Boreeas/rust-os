#![feature(lang_items)]
#![feature(unique)]
#![feature(slice_patterns)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(iter_min_by)]
#![feature(core_intrinsics)]

#![no_std]
#![allow(dead_code)]

extern crate rlibc;
extern crate spin;
extern crate multiboot2;

#[macro_use]
extern crate x86;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod vga_buffer;
mod memory;
mod cpuio;
mod keyboard;
mod cpuid;

use memory::*;
use keyboard::Key::*;
use keyboard::MetaKey::*;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_addr: usize) {
    println!("\n{}               ########################", LIGHT_GRAY);
    println!("               #{}     BorOS v0.0.1     {}#", CYAN, LIGHT_GRAY);
    println!("               ########################");
    
    let boot_info = unsafe { multiboot2::load(multiboot_information_addr) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    // println!("memory areas:");
    // for area in memory_map_tag.memory_areas() {
    // println!("    start: 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
    // }
    //

    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");

    // println!("kernel sections:");
    // for section in elf_sections_tag.sections() {
    // println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
    // section.addr, section.size, section.flags);
    // }
    //

    let kernel_start = elf_sections_tag.sections().map(|sect| sect.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|sect| sect.addr + sect.size).max().unwrap();
    let kernel_len_kb = (kernel_end - kernel_start) / 1024;
    set_color!(LIGHT_GRAY);
    println!("{}Kernel Space:    {}{:#x} {}~{} {:#x} [{}{:4} KiB{}]",
        LIGHT_GRAY, WHITE, kernel_start, 
        LIGHT_GRAY, WHITE, kernel_end,
        GREEN, kernel_len_kb, LIGHT_GRAY);

    let multiboot_start = multiboot_information_addr;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    let multiboot_len_kb = boot_info.total_size / 1024;
    println!("{}Multiboot Space: {}{:#x} {}~{} {:#x} [{}{:4} KiB{}]",
        LIGHT_GRAY, WHITE, multiboot_start, 
        LIGHT_GRAY, WHITE, multiboot_end,
        GREEN, multiboot_len_kb, LIGHT_GRAY);


    let mut alloc = AreaFrameAllocator::new(kernel_start as usize,
                                            kernel_end as usize,
                                            multiboot_start,
                                            multiboot_end,
                                            cpuio::APIC_ADDRESS_BASE,
                                            memory_map_tag.memory_areas());

    
    cpuio::setup_apic(&mut alloc);


    loop {
        match keyboard::next_key() {
            Meta(Esc) | Char('q') => {
                println!("{}> quit", LIGHT_GRAY);
                println!("{}Until next time!", CYAN);
                break;
            }
            Char('v') => {
                println!("{}> vendor", LIGHT_GRAY);
                println!("{}CPU Vendor is {:?}", 
                    CYAN, cpuid::get_vendor());
            }
            Char('f') => {
                println!("{}> features", LIGHT_GRAY);
                println!("{}CPU Features are {:?}", 
                    CYAN, cpuid::get_features());
            }
            Char('t') => {
                println!("{}> trigger", LIGHT_GRAY);
                println!("{}Triggering breakpoint", CYAN);

                unsafe { int!(3); }
            }
            Char('z') => {
                println!("{}> trigger2", LIGHT_GRAY);
                println!("{}Triggering page fault", CYAN);

                unsafe { *(0xdeadbeef as *mut _) = 42 }
            }
            _ => {}
            // Meta(mk) => println!("Meta Key: {:?}", mk),
            // Char(c)  => println!("Char Read: {:?}", c)
        }
    }

}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {
    set_color!(RED);
    println!("\n\neh personality called");
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    set_color!(RED);
    println!("\n\n{}PANIC in {}{}{} at line {}{}{}:",
        RED, 
        LIGHT_GRAY, file, RED,
        LIGHT_GRAY, line, RED);
    println!("    {}", fmt);
    loop {}
}