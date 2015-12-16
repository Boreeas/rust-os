global internal_cpuid

section .text
bits 64

; params:
;    rdi - cpuid call 
;    rsi - pointer to a continous 16 bytes of memory
;          which will be filled
internal_cpuid:
	; save frame pointer and callee-save ebx
	push rbp
	mov rbp, rsp
	push rbx

	; cpuid call - 0 = vendor id
	mov eax, edi
	cpuid

	mov dword [rsi],    eax
	mov dword [rsi+4],  ebx
	mov dword [rsi+8],  ecx
	mov dword [rsi+12], edx

	pop rbx
	pop rbp

	ret