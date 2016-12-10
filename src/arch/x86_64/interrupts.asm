global JUMP_LOCATIONS
global IDT_INFO
global idt
global do_stuff

extern introffset
extern introffset_low_bits
extern introffset_mid_bits
extern introffset_high_bits

struc IdtEntry
    .offset_low     resw 1
    .selector       resw 1
        .text_segment equ 0x08
    .zero1          resb 1
    .attributes     resb 1
        .task_gate_32 equ 0x05
        .intr_gate_16 equ 0x06
        .trap_gate_16 equ 0x07
        .intr_gate_32 equ 0x0e
        .trap_gate_32 equ 0x0f
        .storage_segm equ 1 << 4 
        .present      equ 1 << 7
        ; access
            .ring1    equ 1 << 5
            .ring2    equ 1 << 6
            .ring3    equ 1 << 5 | 1 << 6
    .offset_middle  resw 1
    .offset_high    resd 1
    .zero2          resd 1
endstruc

%macro MK_IDT_ENTRY 1
    istruc IdtEntry
        at IdtEntry.offset_low,     dw introffset_low_bits + intr_handler_%1.length * %1
        at IdtEntry.selector,       dw IdtEntry.text_segment
        at IdtEntry.zero1,          db 0
        at IdtEntry.attributes,     db IdtEntry.present | IdtEntry.intr_gate_32
        at IdtEntry.offset_middle,  dw introffset_mid_bits
        at IdtEntry.offset_high,    dd introffset_high_bits
        at IdtEntry.zero2,          dd 0
    iend
%endmacro

%macro MK_RING3_IDT_ENTRY 1
    istruc IdtEntry
        at IdtEntry.offset_low,     dw introffset_low_bits + intr_handler_%1.length * %1
        at IdtEntry.selector,       dw IdtEntry.text_segment
        at IdtEntry.zero1,          db 0
        at IdtEntry.attributes,     db IdtEntry.ring3 | IdtEntry.present | IdtEntry.intr_gate_32
        at IdtEntry.offset_middle,  dw introffset_mid_bits
        at IdtEntry.offset_high,    dd introffset_high_bits
        at IdtEntry.zero2,          dd 0
    iend
%endmacro

%macro MK_INTR_HANDLER 1
    intr_handler_%1:
        .begin:
        push rbp
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push rsi
        push rdi
        push rdx
        push rcx
        push rbx
        push rax

        ;mov rdi, %1
        ;call qword [jump_locs + 8 * %1]

        pop rax
        pop rbx
        pop rcx
        pop rdx
        pop rdi
        pop rsi
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15
        pop rbp
        iretq
    .end:
    .length equ .end - .begin
%endmacro







; Handlers here
section .interrupt_handlers
bits 64

%assign i 0
%rep 256
    MK_INTR_HANDLER i
%assign i i+1
%endrep





section .text
do_stuff:
    int 128
    ret


; IDT here
section .rodata.intr
IDT_INFO:
    dw end_idt - idt - 1
    dq idt

idt:
%assign i 0
%rep 128
    MK_IDT_ENTRY i
%assign i i+1
%endrep

;Syscall - callable from ring3
MK_RING3_IDT_ENTRY i
%assign i i+1

%rep 127
    MK_IDT_ENTRY i
%assign i i+1
%endrep
end_idt:







; extended handler routine locations here

section .data.intr
JUMP_LOCATIONS: ; for external use only
jump_locs:
%rep 256
dd default_handler ; Handler actual locations
%endrep




; default extended handler routine: crash and print
section .text.intr
bits 64

default_handler:
    ;prints "Err: Unhandled INTR "
    mov rax, 0x043a047204720445
    mov [0xb8000], rax
    mov rax, 0x0468046e04550420
    mov [0xb8008], rax
    mov rax, 0x046c0464046e0461
    mov [0xb8010], rax
    mov rax, 0x0449042004640465
    mov [0xb8018], rax
    mov rax, 0x042004520454044e
    mov [0xb8020], rax

    ;encode INTR number in ascii
    mov rcx, 0x0400
    mov rbx, 10
    mov rax, rdi
    xor rdx, rdx
    div rbx ; rdx = rax % 10 
    or rcx, rdx
    shl rcx, 8
    or rcx, 0x0400

    ;second digit
    xor rdx, rdx
    div rbx ; rdx = rax % 10 
    or rcx, rdx
    shl rcx, 8
    or rcx, 0x0400

    ;third digit
    xor rdx, rdx
    div rbx ; rdx = rax % 10 
    or rcx, rdx
    shl rcx, 8
    or rcx, 0x0400

    mov [0xb802b], rcx

    hlt
    jmp default_handler