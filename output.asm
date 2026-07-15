format PE64 console
entry start

section '.data' data readable writeable
  fmt_int db '%lld', 13, 10, 0
  fmt_str db '%s', 13, 10, 0
  fmt_int_in db '%lld', 0
  fmt_str_in db '%s', 0
  bounds_err_msg db 'Error: Array index out of bounds.', 13, 10, 0
  var_my_list dq 0

section '.text' code readable executable
start:
  sub rsp, 40

  mov rcx, 48
  call [malloc]
  mov qword [rax + 0], 4
  mov qword [rax + 8], 3
  push rax
  mov rax, 10
  pop r10
  mov [r10 + 16 + 8 * 0], rax
  push r10
  mov rax, 20
  pop r10
  mov [r10 + 16 + 8 * 1], rax
  push r10
  mov rax, 30
  pop r10
  mov [r10 + 16 + 8 * 2], rax
  push r10
  pop rax
  mov [var_my_list], rax
  mov rax, [var_my_list]
  push rax
  mov rax, 3
  pop r10
  cmp rax, 0
  jl .L_bounds_error
  mov r11, [r10 + 8]
  cmp rax, r11
  jge .L_bounds_error
  mov rax, [r10 + 16 + 8 * rax]
  mov rdx, rax
  mov rcx, fmt_int
  call [printf]
  mov rcx, 0
  call [ExitProcess]

.L_bounds_error:
  sub rsp, 40
  mov rdx, bounds_err_msg
  mov rcx, fmt_str
  call [printf]
  mov rcx, 1
  call [ExitProcess]

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
    dq rva msvcrt_malloc
    dq rva msvcrt_realloc
    dq 0

  msvcrt_address:
    printf      dq rva msvcrt_printf
    scanf       dq rva msvcrt_scanf
    strlen      dq rva msvcrt_strlen
    rand        dq rva msvcrt_rand
    malloc      dq rva msvcrt_malloc
    realloc     dq rva msvcrt_realloc
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
  msvcrt_malloc        dw 0
                       db 'malloc', 0
  msvcrt_realloc       dw 0
                       db 'realloc', 0

