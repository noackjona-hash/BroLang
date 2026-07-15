format PE64 console
entry start

section '.data' data readable writeable
  fmt_int db '%lld', 13, 10, 0
  fmt_str db '%s', 13, 10, 0
  str_lit_0 db 'Hello from BroLang!', 0
  var_message dq 0
  var_counter dq 0

section '.text' code readable executable
start:
  sub rsp, 40

  mov rax, 5
  mov [var_counter], rax
.L_cond_0:
  mov rax, [var_counter]
  push rax
  mov rax, 0
  pop r10
  cmp r10, rax
  setg al
  movzx rax, al
  cmp rax, 0
  je .L_end_0
  mov rax, [var_counter]
  mov rdx, rax
  mov rcx, fmt_int
  call [printf]
  mov rax, [var_counter]
  push rax
  mov rax, 1
  pop r10
  sub r10, rax
  mov rax, r10
  mov [var_counter], rax
  jmp .L_cond_0
.L_end_0:
  mov rax, str_lit_0
  mov [var_message], rax
  mov rax, [var_message]
  mov rdx, rax
  mov rcx, fmt_str
  call [printf]
  mov rcx, 0
  call [ExitProcess]

section '.idata' import data readable
  library kernel32, 'KERNEL32.DLL', \
          msvcrt, 'MSVCRT.DLL'

  import kernel32, \
         ExitProcess, 'ExitProcess'

  import msvcrt, \
         printf, 'printf'
