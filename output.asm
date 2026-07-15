format PE64 console
entry start

section '.data' data readable writeable
  fmt_int db '%lld', 13, 10, 0
  fmt_str db '%s', 13, 10, 0
  fmt_int_in db '%lld', 0
  fmt_str_in db '%s', 0

section '.text' code readable executable
start:
  sub rsp, 40

  mov rax, 6
  push rax
  pop rcx
  call fn_fib
  mov rdx, rax
  mov rcx, fmt_int
  call [printf]
  mov rcx, 0
  call [ExitProcess]

fn_fib:
  push rbp
  mov rbp, rsp
  sub rsp, 48
  mov [rbp + 16], rcx
  mov rax, [rbp + 16]
  push rax
  mov rax, 1
  pop r10
  cmp r10, rax
  setle al
  movzx rax, al
  cmp rax, 0
  je .L_end_0
  mov rax, [rbp + 16]
  jmp .L_epilogue_fib
.L_end_0:
  mov rax, [rbp + 16]
  push rax
  mov rax, 1
  pop r10
  sub r10, rax
  mov rax, r10
  push rax
  pop rcx
  call fn_fib
  mov [rbp - 8], rax
  mov rax, [rbp + 16]
  push rax
  mov rax, 2
  pop r10
  sub r10, rax
  mov rax, r10
  push rax
  pop rcx
  call fn_fib
  mov [rbp - 16], rax
  mov rax, [rbp - 8]
  push rax
  mov rax, [rbp - 16]
  pop r10
  add rax, r10
  jmp .L_epilogue_fib
.L_epilogue_fib:
  mov rsp, rbp
  pop rbp
  ret

section '.idata' import data readable

  dd rva kernel32_lookup, 0, 0, rva kernel32_name, rva kernel32_address
  dd rva msvcrt_lookup, 0, 0, rva msvcrt_name, rva msvcrt_address
  dd 0, 0, 0, 0, 0

  kernel32_lookup:
    dq rva kernel32_ExitProcess
    dq rva kernel32_Sleep
    dq 0

  kernel32_address:
    ExitProcess dq rva kernel32_ExitProcess
    Sleep       dq rva kernel32_Sleep
    dq 0

  msvcrt_lookup:
    dq rva msvcrt_printf
    dq rva msvcrt_scanf
    dq rva msvcrt_strlen
    dq rva msvcrt_rand
    dq 0

  msvcrt_address:
    printf      dq rva msvcrt_printf
    scanf       dq rva msvcrt_scanf
    strlen      dq rva msvcrt_strlen
    rand        dq rva msvcrt_rand
    dq 0

  kernel32_name db 'KERNEL32.DLL', 0
  msvcrt_name   db 'MSVCRT.DLL', 0

  kernel32_ExitProcess dw 0
                       db 'ExitProcess', 0
  kernel32_Sleep       dw 0
                       db 'Sleep', 0

  msvcrt_printf        dw 0
                       db 'printf', 0
  msvcrt_scanf         dw 0
                       db 'scanf', 0
  msvcrt_strlen        dw 0
                       db 'strlen', 0
  msvcrt_rand          dw 0
                       db 'rand', 0

