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
use vga_buffer::Color::*;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_addr: usize) {
    println!("\n\\{};               ########################", LIGHT_GRAY as u8);
    println!("               #\\{};     BorOS v0.0.1     \\{};#", CYAN as u8, LIGHT_GRAY as u8);
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
    println!("\\{};Kernel Space:    \\{};{:#x} \\{};~\\{}; {:#x} [\\{};{:4} KiB\\{};]",
        LIGHT_GRAY as u8, WHITE as u8, kernel_start, 
        LIGHT_GRAY as u8, WHITE as u8, kernel_end,
        GREEN as u8, kernel_len_kb, LIGHT_GRAY as u8);

    let multiboot_start = multiboot_information_addr;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    let multiboot_len_kb = boot_info.total_size / 1024;
    println!("\\{};Multiboot Space: \\{};{:#x} \\{};~\\{}; {:#x} [\\{};{:4} KiB\\{};]",
        LIGHT_GRAY as u8, WHITE as u8, multiboot_start, 
        LIGHT_GRAY as u8, WHITE as u8, multiboot_end,
        GREEN as u8, multiboot_len_kb, LIGHT_GRAY as u8);


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
                println!("\\{};> quit", LIGHT_GRAY as u8);
                println!("\\{};Until next time!", CYAN as u8);
                break;
            }
            Char('v') => {
                println!("\\{};> vendor", LIGHT_GRAY as u8);
                println!("\\{};CPU Vendor is {:?}", 
                    CYAN as u8, cpuid::get_vendor());
            }
            Char('f') => {
                println!("\\{};> features", LIGHT_GRAY as u8);
                println!("\\{};CPU Features are {:?}"
                    , CYAN as u8, cpuid::get_features());
            }
            Char('t') => {
                println!("\\{};> trigger", LIGHT_GRAY as u8);
                println!("\\{};Triggering breakpoint", CYAN as u8);

                unsafe { int!(3); }
            }
            Char('z') => {
                println!("\\{};> trigger2", LIGHT_GRAY as u8);
                println!("\\{};Triggering page fault", CYAN as u8);

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
    println!("\n\n\\{};PANIC in \\{};{}\\{}; at line \\{};{}\\{};:",
        RED as u8, 
        LIGHT_GRAY as u8, file, RED as u8,
        LIGHT_GRAY as u8, line, RED as u8);
    println!("    {}", fmt);
    loop {}
}