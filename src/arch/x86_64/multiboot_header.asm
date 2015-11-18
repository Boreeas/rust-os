%define magic 0xe85250d6
%define arch  0	; x86
%define length header_end - header_start

section .multiboot_header
header_start:
	dd magic
	dd arch
	dd length
	dd 0x100000000 - (magic + arch + length)

	; end tag
	dw 0 ; tag type
	dw 0 ; tag flags
	dd 8 ; tag len
header_end: