format PE64 console
entry start

section '.data' data readable writeable
  fmt_int db '%lld', 13, 10, 0
  fmt_str db '%s', 13, 10, 0
  fmt_int_in db '%lld', 0
  fmt_str_in db '%s', 0
  var_z dq 0

section '.text' code readable executable
start:
  sub rsp, 40

  mov rax, 12
  push rax
  mov rax, 18
  push rax
  pop rdx
  pop rcx
  call addiere
  mov [var_z], rax
  mov rax, [var_z]
  mov rdx, rax
  mov rcx, fmt_int
  call [printf]
  mov rcx, 0
  call [ExitProcess]

addiere:
  push rbp
  mov rbp, rsp
  sub rsp, 48
  mov [rbp + 16], rcx
  mov [rbp + 24], rdx
  mov rax, [rbp + 16]
  push rax
  mov rax, [rbp + 24]
  pop r10
  add rax, r10
  mov [rbp - 8], rax
  mov rax, [rbp - 8]
  jmp .L_epilogue_addiere
.L_epilogue_addiere:
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

