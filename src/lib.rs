#![feature(no_std, lang_items, unique, core_str_ext, const_fn, iter_cmp)]
#![no_std]
extern crate rlibc;
extern crate spin;
extern crate multiboot2;
mod vga_buffer;
mod memory;

use core::ptr::Unique;
use vga_buffer::WRITER as vga;
use memory::*;


#[no_mangle]
pub extern fn rust_main(multiboot_information_addr: usize) {
	println!("");
	set_color!(LIGHT_GRAY);
	println!("               ########################");
	print!("               #");
	set_color!(CYAN);
	print!("     BorOS v0.0.1     ");
	set_color!(LIGHT_GRAY);
	println!("#");
	println!("               ########################");

	let boot_info = unsafe{ multiboot2::load(multiboot_information_addr) };
	let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
	
	/*
	println!("memory areas:");
	for area in memory_map_tag.memory_areas() {
	    println!("    start: 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
	}
	*/

	let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");

	/*
	println!("kernel sections:");
	for section in elf_sections_tag.sections() {
	    println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
	        section.addr, section.size, section.flags);
	}
	*/

	let kernel_start = elf_sections_tag.sections().map(|sect| sect.addr).min().unwrap();
	let kernel_end = elf_sections_tag.sections().map(|sect| sect.addr + sect.size).max().unwrap();
	let kernel_len_kb = (kernel_end - kernel_start) / 1024;
	set_color!(LIGHT_GRAY);
	print!("Kernel Space:    ");
	set_color!(WHITE);
	print!("0x{:x}", kernel_start);
	set_color!(LIGHT_GRAY);
	print!(" ~ ");
	set_color!(WHITE);
	print!("0x{:x}", kernel_end);
	set_color!(LIGHT_GRAY);
	print!(" [");
	set_color!(GREEN);
	print!("{:4} KiB", kernel_len_kb);
	set_color!(LIGHT_GRAY);
	println!("]");

	let multiboot_start = multiboot_information_addr;
	let multiboot_end = multiboot_start + (boot_info.total_size as usize);
	let multiboot_len_kb = boot_info.total_size / 1024;
	set_color!(LIGHT_GRAY);
	print!("Multiboot Space: ");
	set_color!(WHITE);
	print!("0x{:x}", multiboot_start);
	set_color!(LIGHT_GRAY);
	print!(" ~ ");
	set_color!(WHITE);
	print!("0x{:x}", multiboot_end);
	set_color!(LIGHT_GRAY);
	print!(" [");
	set_color!(GREEN);
	print!("{:4} KiB", multiboot_len_kb);
	set_color!(LIGHT_GRAY);
	println!("]");

	let mut alloc = memory::AreaFrameAllocator::new(
		kernel_start as usize, kernel_end as usize, 
		multiboot_start, multiboot_end,
		memory_map_tag.memory_areas()
	);
}

#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
	set_color!(RED);
    print!("\n\nPANIC in ");
    set_color!(LIGHT_GRAY);
    print!("{}", file);
    set_color!(RED);
    print!(" at line ");
    set_color!(LIGHT_GRAY);
    print!("{}", line);
    set_color!(RED);
    println!(":\n    {}", fmt);
    loop{}
}
