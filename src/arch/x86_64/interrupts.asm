global interrupt_handler

section .text
bits 64

interrupt_handler:
    ;mov dword [0xfee000b0], 0 ; Acknowledge EOI
    iretq
