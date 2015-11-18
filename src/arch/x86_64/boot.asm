%define VGA_BUF 0xb8000

global start
extern long_mode_start

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    resb 4096
stack_top:


section .rodata
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; code segment
.data: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41) ; data segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64


section .text
bits 32
start:
    mov esp, stack_top
    mov edi, ebx

    call test_multiboot
    call test_cpuid
    call test_long_mode

    call setup_page_tables
    call enable_paging

    lgdt [gdt64.pointer] 

    ; update selectors
    mov ax, gdt64.data
    mov ss, ax  ; stack selector
    mov ds, ax  ; data selector
    mov es, ax  ; extra selector]

    jmp gdt64.code:long_mode_start

    mov dword [VGA_BUF], 0x2f4b2f4f
    hlt






setup_page_tables:
    mov eax, p3_table
    or  eax, 0b11 ; present, writable
    mov [p4_table], eax

    mov eax, p2_table
    or  eax, 0b11
    mov [p4_table], eax

    mov ecx, 0
.map_p2_tables:
    mov eax, 0x20000
    mul ecx
    or  eax, 0b10000011 ; huge, present, writable
    mov [p2_table + ecx*8], eax

    inc ecx
    cmp ecx, 512
    jne .map_p2_tables

    ret



enable_paging:
    ; load P4 to cr3 register (cpu uses this to access the P4 table)
    mov eax, p4_table
    mov cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    or eax, 1 << 16
    mov cr0, eax

    ret









test_multiboot:
    cmp eax, 0x36d76289; multiboot magic number
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0"
    jmp error

test_cpuid:
    pushfd               ; Store the FLAGS-register.
    pop eax              ; Restore the A-register.
    mov ecx, eax         ; Set the C-register to the A-register.
    xor eax, 1 << 21     ; Flip the ID-bit, which is bit 21.
    push eax             ; Store the A-register.
    popfd                ; Restore the FLAGS-register.
    pushfd               ; Store the FLAGS-register.
    pop eax              ; Restore the A-register.
    push ecx             ; Store the C-register.
    popfd                ; Restore the FLAGS-register.
    xor eax, ecx         ; Do a XOR-operation on the A-register and the C-register.
    jz .no_cpuid         ; The zero flag is set, no CPUID.
    ret                  ; CPUID is available for use.
.no_cpuid:
    mov al, "1"
    jmp error

test_long_mode:
    mov eax, 0x80000000    ; Set the A-register to 0x80000000.
    cpuid                  ; CPU identification.
    cmp eax, 0x80000001    ; Compare the A-register with 0x80000001.
    jb .no_long_mode       ; It is less, there is no long mode.
    mov eax, 0x80000001    ; Set the A-register to 0x80000001.
    cpuid                  ; CPU identification.
    test edx, 1 << 29      ; Test if the LM-bit, which is bit 29, is set in the D-register.
    jz .no_long_mode       ; They aren't, there is no long mode.
    ret
.no_long_mode:
    mov al, "2"
    jmp error




; display 'ERR: X' on screen where X is an error code
; @param al The error code in ascii
error:
    mov dword [VGA_BUF], 0x4f524f45
    mov dword [VGA_BUF + 4], 0x4f3a4f52
    mov dword [VGA_BUF + 8], 0x4f204f20
    mov byte  [VGA_BUF + 11], al
    hlt