arch ?= x86_64
var ?= foo
target ?= x86_64-unknown-none-gnu
rust_os := target/$(target)/debug/libos.a
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

.SUFFIXES:


.PHONY: all clean run iso

all: $(kernel)-release

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -hda $(iso) -s -d int -no-reboot

run-release: $(iso)-release
	@qemu-system-x86_64 -hda $(iso)

debug: $(iso)
	@qemu-system-x86_64 -hda $(iso) -s -S -d int -no-reboot

gdb:
    @rust-os-gdb/bin/rust-gdb "build/kernel-x86_64.bin" -ex "target remote :1234"


show-asm: xargo-asm $(rust_os) $(assembly_object_files) $(linker_script)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(iso)-release: $(kernel)-release $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): xargo $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -Map=ldmap.txt -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

$(kernel)-release: xargo-release $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

libs: build/arch/$(arch)/libcpuid.a

xargo-release: libs
	@xargo rustc --target $(target) --release -- -Z no-landing-pads -L build/arch/$(arch) -lcpuid -linterrupts

xargo: libs
	@xargo rustc --verbose --target $(target) -- --verbose -Z no-landing-pads -L build/arch/$(arch) -lcpuid -linterrupts

xargo-asm:
	@xargo rustc --target $(target) -- -Z no-landing-pads --emit asm

build/arch/$(arch)/lib%.a: build/arch/$(arch)/%.o
	@ar r $@ $<

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@